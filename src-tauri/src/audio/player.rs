use crate::audio::eq::{write_wav_f32, GraphicEq};
use crate::radio::{self, SharedBuffer, StreamReader};
use maudio::audio::sample_rate::SampleRate;
use maudio::data_source::sources::decoder::{Cb, Decoder, DecoderBuilder, DecoderOps, Fs, Owned};
use maudio::engine::{Engine, EngineOps};
use maudio::sound::Sound;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

const STATE_STOPPED: u8 = 0;
const STATE_PLAYING: u8 = 1;
const STATE_PAUSED: u8 = 2;
const NO_SEEK: u32 = f32::to_bits(-1.0);

enum Cmd {
    Ping { result_tx: std::sync::mpsc::Sender<()> },
    Play {
        path: PathBuf,
        handle: AppHandle,
        result_tx: std::sync::mpsc::Sender<Result<(), String>>,
    },
    PlayRadio {
        url: String,
        handle: AppHandle,
    },
    StopRadio,
    Toggle,
    Seek(f32),
    SetVolume(f32),
    SetEqBand { index: u8, gain_db: f32 },
    SetEqEnabled(bool),
    Stop,
}

pub struct Player {
    cmd_tx: mpsc::Sender<Cmd>,
    state: Arc<AtomicU8>,
    position_secs: Arc<AtomicU32>,
    seek_target: Arc<AtomicU32>,
    radio_active: Arc<AtomicBool>,
    _handle: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl Player {
    pub fn new() -> (Self, Arc<AtomicU32>) {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let state = Arc::new(AtomicU8::new(STATE_STOPPED));
        let position_secs = Arc::new(AtomicU32::new(f32::to_bits(0.0)));
        let seek_target = Arc::new(AtomicU32::new(NO_SEEK));
        let volume = Arc::new(AtomicU32::new(f32::to_bits(0.8)));
        let radio_active = Arc::new(AtomicBool::new(false));

        let th_state = Arc::clone(&state);
        let th_pos = Arc::clone(&position_secs);
        let th_seek = Arc::clone(&seek_target);
        let th_vol = Arc::clone(&volume);
        let th_radio = Arc::clone(&radio_active);

        let handle = std::thread::spawn(move || {
            Self::audio_thread(cmd_rx, th_state, th_pos, th_seek, th_vol, th_radio);
        });

        (
            Self {
                cmd_tx,
                state,
                position_secs,
                seek_target,
                radio_active,
                _handle: Mutex::new(Some(handle)),
            },
            volume,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn audio_thread(
        cmd_rx: mpsc::Receiver<Cmd>,
        state: Arc<AtomicU8>,
        pos: Arc<AtomicU32>,
        seek_target: Arc<AtomicU32>,
        vol: Arc<AtomicU32>,
        radio_active: Arc<AtomicBool>,
    ) {
        let Ok(engine) = Engine::new() else {
            eprintln!("[audio] FAILED to create miniaudio engine");
            return;
        };
        let _ = engine.set_volume(f32::from_bits(vol.load(Ordering::Acquire)));

        if engine.device().is_none() {
            eprintln!("[audio] WARNING: engine has NO device (Null/Silent backend)!");
        }

        #[allow(dead_code)]
        enum StoredDecoder {
            Fs(Decoder<f32, Fs>),
            Owned(Decoder<f32, Owned>),
        }
        let mut sound: Option<Sound<'_>> = None;
        let mut handle: Option<AppHandle> = None;
        let mut radio_decoder: Option<Decoder<f32, Cb>> = None;
        let mut local_decoder: Option<StoredDecoder> = None;
        let mut current_path: Option<PathBuf> = None;
        let mut eq = GraphicEq::new();
        let mut radio_dl_handle: Option<JoinHandle<()>> = None;
        let mut radio_shared: Option<Arc<SharedBuffer>> = None;

        loop {
            match cmd_rx.try_recv() {
                Ok(Cmd::Play { path, handle: h, result_tx }) => {
                    Self::clean_radio(
                        &mut sound,
                        &mut radio_decoder, &mut radio_dl_handle,
                        &mut radio_shared, &radio_active, &state, &pos,
                    );
                    local_decoder.take();
                    if let Some(mut old) = sound.take() {
                        let _ = old.stop_sound();
                    }
                    current_path = Some(path.clone());
                    match DecoderBuilder::new_f32(2, SampleRate::Sr44100).from_file(&path) {
                        Err(e) => {
                            let err = format!("Failed to decode file: {e:?}");
                            eprintln!("[audio] {err}");
                            let _ = result_tx.send(Err(err));
                        }
                        Ok(dec) => {
                            match engine.new_sound_from_source(&dec) {
                                Err(e) => {
                                    let err = format!("Failed to create sound: {e:?}");
                                    eprintln!("[audio] {err}");
                                    let _ = result_tx.send(Err(err));
                                }
                                Ok(mut s) => {
                                    let _ = s.play_sound();
                                    local_decoder = Some(StoredDecoder::Fs(dec));
                                    sound = Some(s);
                                    handle = Some(h);
                                    state.store(STATE_PLAYING, Ordering::Release);
                                    let _ = result_tx.send(Ok(()));
                                }
                            }
                        }
                    }
                }
                Ok(Cmd::Ping { result_tx }) => { let _ = result_tx.send(()); }
                Ok(Cmd::PlayRadio { url, handle: h }) => {
                    Self::clean_radio(
                        &mut sound,
                        &mut radio_decoder, &mut radio_dl_handle,
                        &mut radio_shared, &radio_active, &state, &pos,
                    );
                    local_decoder.take();
                    if let Some(mut s) = sound.take() {
                        let _ = s.stop_sound();
                    }

                    let shared = Arc::new(SharedBuffer {
                        inner: std::sync::Mutex::new(radio::RadioBuffer {
                            data: Vec::new(),
                            finished: false,
                        }),
                        cond: std::sync::Condvar::new(),
                    });

                    let dl_stop = Arc::clone(&radio_active);
                    let dl_handle = radio::spawn_downloader(url, Arc::clone(&shared), dl_stop);

                    // Pre-buffer at least 64KB before creating decoder
                    let ready = {
                        let deadline = Instant::now() + Duration::from_secs(30);
                        let mut rb = shared.inner.lock().unwrap();
                        while rb.data.len() < 65536 && !rb.finished && Instant::now() < deadline {
                            rb = shared.cond.wait(rb).unwrap();
                        }
                        !rb.data.is_empty()
                    };

                    let Some(dl_handle) = (if ready { Some(dl_handle) } else {
                        radio_active.store(false, Ordering::Release);
                        drop(dl_handle);
                        eprintln!("Radio stream returned no data");
                        None
                    }) else { continue };

                    let reader = StreamReader {
                        shared: Arc::clone(&shared),
                        pos: 0,
                    };

                    let dec = match DecoderBuilder::new_f32(2, SampleRate::Sr44100)
                        .from_reader(reader)
                    {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("Failed to create radio decoder: {e:?}");
                            radio_active.store(false, Ordering::Release);
                            drop(dl_handle);
                            continue;
                        }
                    };

                    match engine.new_sound_from_source(
                        &dec,
                    ) {
                        Ok(mut s) => {
                            let _ = s.play_sound();
                            sound = Some(s);
                            radio_decoder = Some(dec);
                            radio_dl_handle = Some(dl_handle);
                            radio_shared = Some(shared);
                            handle = Some(h);
                            radio_active.store(true, Ordering::Release);
                            state.store(STATE_PLAYING, Ordering::Release);
                            if let Some(ref h) = handle {
                                let _ = h.emit("radio:playing", true);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to create radio sound: {e:?}");
                            radio_active.store(false, Ordering::Release);
                            drop(dec);
                            drop(dl_handle);
                            continue;
                        }
                    }
                }
                Ok(Cmd::StopRadio) => {
                    Self::clean_radio(
                        &mut sound,
                        &mut radio_decoder, &mut radio_dl_handle,
                        &mut radio_shared, &radio_active, &state, &pos,
                    );
                }
                Ok(Cmd::Toggle) => {
                    if let Some(ref mut s) = sound {
                        if state.load(Ordering::Relaxed) == STATE_PLAYING {
                            let _ = s.stop_sound();
                            state.store(STATE_PAUSED, Ordering::Release);
                        } else {
                            let _ = s.play_sound();
                            state.store(STATE_PLAYING, Ordering::Release);
                        }
                    }
                }
                Ok(Cmd::Seek(secs)) => {
                    if let Some(ref mut s) = sound {
                        let _ = s.seek_to_second(secs);
                    }
                }
                Ok(Cmd::SetVolume(vol)) => {
                    let _ = engine.set_volume(vol);
                }
                Ok(Cmd::SetEqBand { index, gain_db }) => {
                    eq.set_band(index as usize, gain_db);
                    let current_pos = sound.as_ref().and_then(|s| s.cursor_seconds().ok()).unwrap_or(0.0);
                    if let Some(ref path) = current_path {
                        let pcm = Self::read_pcm(path);
                        if let Some(mut pcm) = pcm {
                            eq.process(&mut pcm);
                            let wav_bytes = write_wav_f32(&pcm, 44100);
                            if let Ok(new_dec) = DecoderBuilder::new_f32(2, SampleRate::Sr44100)
                                .copy_memory(wav_bytes)
                            {
                                if let Ok(mut new_s) = engine.new_sound_from_source(&new_dec) {
                                    if let Some(mut old) = sound.take() {
                                        let _ = old.stop_sound();
                                    }
                                    let _ = new_s.seek_to_second(current_pos);
                                    let _ = new_s.play_sound();
                                    local_decoder = Some(StoredDecoder::Owned(new_dec));
                                    sound = Some(new_s);
                                }
                            }
                        }
                    }
                }
                Ok(Cmd::SetEqEnabled(enabled)) => {
                    eq.set_enabled(enabled);
                    let current_pos = sound.as_ref().and_then(|s| s.cursor_seconds().ok()).unwrap_or(0.0);
                    if let Some(ref path) = current_path {
                        let pcm = Self::read_pcm(path);
                        if let Some(mut pcm) = pcm {
                            eq.process(&mut pcm);
                            let wav_bytes = write_wav_f32(&pcm, 44100);
                            if let Ok(new_dec) = DecoderBuilder::new_f32(2, SampleRate::Sr44100)
                                .copy_memory(wav_bytes)
                            {
                                if let Ok(mut new_s) = engine.new_sound_from_source(&new_dec) {
                                    if let Some(mut old) = sound.take() {
                                        let _ = old.stop_sound();
                                    }
                                    let _ = new_s.seek_to_second(current_pos);
                                    let _ = new_s.play_sound();
                                    local_decoder = Some(StoredDecoder::Owned(new_dec));
                                    sound = Some(new_s);
                                }
                            }
                        }
                    }
                }
                Ok(Cmd::Stop) => {
                    Self::clean_radio(
                        &mut sound,
                        &mut radio_decoder, &mut radio_dl_handle,
                        &mut radio_shared, &radio_active, &state, &pos,
                    );
                    local_decoder.take();
                    if let Some(mut s) = sound.take() {
                        let _ = s.stop_sound();
                    }
                    handle = None;
                    current_path = None;
                    state.store(STATE_STOPPED, Ordering::Release);
                    pos.store(f32::to_bits(0.0), Ordering::Release);
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => break,
            }

            if let Some(ref mut s) = sound {
                let sk = seek_target.swap(NO_SEEK, Ordering::Acquire);
                if sk != NO_SEEK {
                    let _ = s.seek_to_second(f32::from_bits(sk));
                }

                if !radio_active.load(Ordering::Relaxed) {
                    if let Ok(p) = s.cursor_seconds() {
                        pos.store(f32::to_bits(p), Ordering::Relaxed);

                    }
                }

                if s.ended() {
                    state.store(STATE_STOPPED, Ordering::Release);
                    if radio_active.load(Ordering::Relaxed) {
                        if let Some(ref h) = handle {
                            let _ = h.emit("radio:ended", ());
                        }
                        Self::clean_radio(
                            &mut sound,
                            &mut radio_decoder, &mut radio_dl_handle,
                            &mut radio_shared, &radio_active, &state, &pos,
                        );
                        sound = None;
                        handle = None;
                    } else {
                        if let Some(ref h) = handle {
                            let _ = h.emit("playback-ended", ());
                        }
                        sound = None;
                        local_decoder = None;
                        current_path = None;
                        handle = None;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(30));
        }
    }

    fn read_pcm(path: &PathBuf) -> Option<Vec<f32>> {
        let mut dec = DecoderBuilder::new_f32(2, SampleRate::Sr44100).from_file(path).ok()?;
        let n = dec.length_pcm().unwrap_or(0);
        let buf = dec.read_pcm_frames(n).ok()?;
        Some(buf.as_ref().to_vec())
    }

    fn clean_radio(
        sound: &mut Option<Sound<'_>>,
        radio_decoder: &mut Option<Decoder<f32, Cb>>,
        radio_dl_handle: &mut Option<JoinHandle<()>>,
        radio_shared: &mut Option<Arc<SharedBuffer>>,
        radio_active: &Arc<AtomicBool>,
        state: &Arc<AtomicU8>,
        pos: &Arc<AtomicU32>,
    ) {
        if !radio_active.load(Ordering::Relaxed) {
            return;
        }
        if let Some(mut s) = sound.take() {
            let _ = s.stop_sound();
        }
        radio_active.store(false, Ordering::Release);
        if let Some(ref shared) = radio_shared {
            shared.cond.notify_all();
        }
        radio_shared.take();
        drop(radio_decoder.take());
        if let Some(jh) = radio_dl_handle.take() {
            let _ = jh.join();
        }
        state.store(STATE_STOPPED, Ordering::Release);
        pos.store(f32::to_bits(0.0), Ordering::Release);
    }

    pub fn ping(&self) -> Result<(), String> {
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        self.cmd_tx
            .send(Cmd::Ping { result_tx })
            .map_err(|_| "Audio player channel closed".to_string())?;
        result_rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .map_err(|_| "Audio player did not respond within 3 seconds".to_string())
    }

    pub fn play_track(&self, path: PathBuf, handle: AppHandle) -> Result<(), String> {
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        self.cmd_tx
            .send(Cmd::Play { path, handle, result_tx })
            .map_err(|_| "Audio player channel closed".to_string())?;
        result_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .map_err(|_| "Audio player did not respond within 10 seconds".to_string())?
    }

    pub fn play_radio(&self, url: String, handle: AppHandle) {
        let _ = self.cmd_tx.send(Cmd::PlayRadio { url, handle });
    }

    pub fn stop_radio(&self) {
        let _ = self.cmd_tx.send(Cmd::StopRadio);
    }

    pub fn toggle(&self) -> bool {
        let current = self.state.load(Ordering::Relaxed);
        let _ = self.cmd_tx.send(Cmd::Toggle);
        current == STATE_PAUSED
    }

    pub fn stop(&self) {
        let _ = self.cmd_tx.send(Cmd::Stop);
    }

    pub fn seek(&self, pos_secs: f32) {
        self.seek_target
            .store(f32::to_bits(pos_secs), Ordering::Release);
        let _ = self.cmd_tx.send(Cmd::Seek(pos_secs));
    }

    pub fn set_volume(&self, vol: f32) {
        let _ = self.cmd_tx.send(Cmd::SetVolume(vol));
    }

    pub fn set_eq_band(&self, index: u8, gain_db: f32) {
        let _ = self.cmd_tx.send(Cmd::SetEqBand { index, gain_db });
    }

    pub fn set_eq_enabled(&self, enabled: bool) {
        let _ = self.cmd_tx.send(Cmd::SetEqEnabled(enabled));
    }

    pub fn position(&self) -> f32 {
        f32::from_bits(self.position_secs.load(Ordering::Relaxed))
    }

    pub fn is_playing(&self) -> bool {
        self.state.load(Ordering::Relaxed) == STATE_PLAYING
    }

    pub fn is_radio_active(&self) -> bool {
        self.radio_active.load(Ordering::Relaxed)
    }
}
