use ringbuf::HeapRb;

fn main() {
    let ringbuffer_left = HeapRb::<f32>::new(20);

    let in_a_p: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let extra: Vec<f32> = vec![9.0, 10.0, 11.0];
    let (mut ringbuffer_left_in, mut ringbuffer_left_out) = ringbuffer_left.split();
    ringbuffer_left_in.push_iter(&mut in_a_p.into_iter());

    let mut index = 0;
    while !ringbuffer_left_out.is_empty() {
        if index == 3 {
            let _ = ringbuffer_left_in.push(12.0);
        }
        let (older_audio, newer_audio) = ringbuffer_left_out.as_slices();
        if index == 2 {
            ringbuffer_left_in.push_iter(&mut extra.iter().copied());
        }
        println!("rb: {:?}  {:?}", older_audio, newer_audio);
        ringbuffer_left_out.skip(2);
        index += 1;
    }
}
