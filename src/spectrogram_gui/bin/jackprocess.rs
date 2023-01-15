use jack;

use ringbuf::Producer;
use ringbuf::SharedRb;
use std::mem::MaybeUninit;
use std::sync::Arc;
use std::{thread, time::Duration};

pub fn start_jack_thread(
    mut ringbuffer_left_in: Producer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>,
    mut ringbuffer_right_in: Producer<f32, Arc<SharedRb<f32, std::vec::Vec<MaybeUninit<f32>>>>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut run: bool = true;
        let (client, _status) =
            jack::Client::new("spectrogram_gui", jack::ClientOptions::NO_START_SERVER)
                .expect("No Jack server running\n");

        let sample_rate = client.sample_rate();
        // register ports
        let in_a = client
            .register_port("spectrogram_gui_l", jack::AudioIn::default())
            .unwrap();
        let in_b = client
            .register_port("spectrogram_gui_r", jack::AudioIn::default())
            .unwrap();
        let frame_size = client.buffer_size() as usize;
        let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
            let in_a_p = in_a.as_slice(ps);
            let in_b_p = in_b.as_slice(ps);

            ringbuffer_left_in.push_iter(&mut in_a_p.into_iter().map(|x| *x));
            ringbuffer_right_in.push_iter(&mut in_b_p.into_iter().map(|x| *x));
            jack::Control::Continue
        };
        let process = jack::ClosureProcessHandler::new(process_callback);
        let active_client = client.activate_async((), process).unwrap();

        while run {
            thread::sleep(Duration::from_millis(100));
            /*
            match rx_close.recv() {
                Ok(running) => run = running,
                Err(_) => run = false,
            }*/
        }
        match active_client.deactivate() {
            Ok(_) => println!("exit audio thread\n"),
            Err(_) => println!("exit audio thread,client deactivation err\n"),
        }
    })
}
