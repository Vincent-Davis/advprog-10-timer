use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

/// Shared state antara future dan thread timer.
struct SharedState {
    /// Apakah sleep time sudah lewat?
    completed: bool,
    /// Waker untuk membangunkan task ketika timer selesai.
    waker: Option<Waker>,
}

/// Future yang selesai setelah durasi tertentu.
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

impl TimerFuture {
    /// Buat TimerFuture yang akan siap setelah `duration` berlalu.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        // Clone state untuk thread
        let thread_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration);
            let mut state = thread_state.lock().unwrap();
            state.completed = true;
            // setelah completed, panggil wake() jika waker sudah di-set
            if let Some(waker) = state.waker.take() {
                waker.wake();
            }
        });

        TimerFuture { shared_state }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.shared_state.lock().unwrap();
        if state.completed {
            // timer sudah selesai
            Poll::Ready(())
        } else {
            // simpan waker terbaru agar thread bisa membangunkan task ini
            // waker perlu di-update tiap kali poll karena task bisa berpindah
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
