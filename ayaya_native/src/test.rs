#[cfg(test)]
mod tests {
    use crate::splitting::{SplittedFrame, generate_index_cache};
    use rand::Rng;
    use crate::colorlib::transform_frame_to_mc;

//Width                                    : 3840 pixels
//Height                                   : 2160 pixels

//8294400



    #[test]
    fn cpu_fast_split_test(){
        let width: u32 = 3840;
        let height: u32 = 2160;

        let mut splitted_frames = SplittedFrame::initialize_frames(width as i32, height as i32).unwrap();
        let mut data = Vec::<u8>::with_capacity((width * height * 3) as usize);
        let mut rng = rand::thread_rng();

        data.resize_with(data.capacity(), || rng.gen());

        let color_data = transform_frame_to_mc(&data, width, height);

        //pre test
        let now = std::time::Instant::now();
        let normal_split = SplittedFrame::split_frames(&color_data, &mut splitted_frames, width as i32).expect("Couldn't do normal split'");
        let elapsed = now.elapsed();
        println!("Elapsed for normal: {:.2?}", elapsed);

        //main testst
        let fast_cache = generate_index_cache(width, height, splitted_frames);
        let now = std::time::Instant::now();

        let fast_split = SplittedFrame::fast_split(&color_data, fast_cache);

        let elapsed = now.elapsed();
        println!("Elapsed for fast: {:.2?}", elapsed);

        let mut diff = 0;
        for i in 0..fast_split.len()-1 {
            if normal_split[i] != fast_split[i] {
                diff += 1;
                println!("O: {}, THE: {}", fast_split[i], normal_split[i])
            }
        };

        println!("D: {}", diff);
    }
}
