# Saya Architecture

## Thread Model

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           TOKIO RUNTIME (4 workers)                          │
│                                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────────┐  │
│  │   event_loop    │  │    ui_loop      │  │        watcher_io           │  │
│  │     (async)     │  │    (async)      │  │          (async)            │  │
│  │                 │  │                 │  │                             │  │
│  │ - recv events   │  │ - spawn UI thrd │  │ - hotkey listener           │  │
│  │ - handle OCR    │  │ - forward msgs  │  │ - clipboard/websocket       │  │
│  │ - lookup dict   │  │                 │  │                             │  │
│  └────────┬────────┘  └────────┬────────┘  └─────────────────────────────┘  │
│           │                    │                                             │
└───────────┼────────────────────┼─────────────────────────────────────────────┘
            │                    │
            │                    │ std::thread::spawn
            │                    ▼
            │  ┌───────────────────────────────────────────────────────────────┐
            │  │                    UI THREAD (std::thread)                    │
            │  │                                                               │
            │  │  ┌─────────────────────────────┐  ┌───────────────────────┐  │
            │  │  │      Slint Event Loop       │  │    SLINT-RX Thread    │  │
            │  │  │                             │  │    (std::thread)      │  │
            │  │  │  - window rendering         │  │                       │  │
            │  │  │  - button callbacks         │  │  - recv from app      │  │
            │  │  │  - invoke_from_event_loop() │◄─│  - invoke_from_...()  │  │
            │  │  │                             │  │                       │  │
            │  │  └─────────────────────────────┘  └───────────────────────┘  │
            │  │                                                               │
            │  └───────────────────────────────────────────────────────────────┘
            │
            ▼
    ┌───────────────────┐
    │  spawn_blocking   │  (OCR capture, COM/WinRT operations)
    │   thread pool     │
    └───────────────────┘
```

## Channel Flow

```
                          ASYNC CHANNELS (kanal)
    ┌─────────────────────────────────────────────────────────────────┐
    │                                                                 │
    │   ui_to_app_tx ──────────────────────► ui_to_app_rx            │
    │   (AsyncSender)                        (AsyncReceiver)          │
    │        ▲                                     │                  │
    │        │                                     ▼                  │
    │   [ui_loop]                            [event_loop]             │
    │        │                                     │                  │
    │        ▼                                     ▼                  │
    │   app_to_ui_rx ◄────────────────────── app_to_ui_tx            │
    │   (AsyncReceiver)                      (AsyncSender)            │
    │                                                                 │
    └─────────────────────────────────────────────────────────────────┘

                          SYNC CHANNELS (kanal)
    ┌─────────────────────────────────────────────────────────────────┐
    │                                                                 │
    │   sync_tx ───────────────────────────► sync_rx.as_async()      │
    │   (Sender)                             (AsyncReceiver)          │
    │      ▲                                       │                  │
    │      │ Slint callbacks                       │ forward to       │
    │      │ (button click)                        ▼ ui_to_app_tx     │
    │                                                                 │
    │   app_sync_rx ◄────────────────────── app_sync_tx              │
    │   (Receiver)                           (Sender)                 │
    │      │                                       ▲                  │
    │      │ SLINT-RX thread                       │ forward from     │
    │      ▼ invoke_from_event_loop()              │ app_to_ui_rx     │
    │                                                                 │
    └─────────────────────────────────────────────────────────────────┘
```

## Event Flow Example: OCR Capture Button

```
[1] Button Click (Slint callback, sync)
         │
         ▼ sync_tx.send()
[2] sync_rx.as_async().recv()  (ui_loop, async)
         │
         ▼ ui_to_app_tx.send()
[3] ui_to_app_rx.recv()  (event_loop, async)
         │
         ▼ tokio::spawn_blocking
[4] OCR capture + recognize  (blocking thread pool)
         │
         ▼ app_to_ui_tx.send()
[5] app_to_ui_rx.recv()  (forward_to_ui task, async)
         │
         ▼ app_sync_tx.send()
[6] app_sync_rx.recv()  (SLINT-RX thread, blocking)
         │
         ▼ slint::invoke_from_event_loop()
[7] UI Update  (Slint main thread)
```

## Key Design Decisions

| Problem | Solution |
|---------|----------|
| Slint event loop blocks tokio workers | Run Slint in dedicated `std::thread` |
| `slint::spawn_local` can't drive tokio futures | Use sync channels + `tokio::spawn` bridge |
| Slint callbacks are sync, need async send | Sync `Sender` → async forwarder task |
| UI updates must happen on Slint thread | `slint::invoke_from_event_loop()` |
| OCR/COM requires thread-local init | `tokio::spawn_blocking` with `CoInitializeEx` |
