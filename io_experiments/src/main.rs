use facto_loop_miner_common::log_init_trace;
use std::path::PathBuf;
use tracing::info;

fn main() {
    println!("asd");
    log_init_trace();

    tracing::debug!("hello io_experiment");

    let file_path: PathBuf = match 2 {
        1 => "/xf-megafile/data/pages.db.index",
        2 => "README.md",
        3 => {
            "/home/desk/IdeaProjects/facto-loop-miner/work/out0/step00-import/pixel-xy-indexes.dat"
        }
        4 => "/home/desk/IdeaProjects/facto-loop-miner/work/out0/step10-base/pixel-xy-indexes.dat",
        5 => "/hugetemp/pixel-xy-indexes.dat",
        _ => unimplemented!("fuck"),
    }
    .into();
    info!("io_experiments processing {}", file_path.display());

    inner::inner_main()
}

#[cfg(not(feature = "uring"))]
mod inner {
    pub fn inner_main() {
        unimplemented!("must enable feature")
    }
}

#[cfg(feature = "uring")]
mod inner {
    use facto_loop_miner_common::LOCALE;
    use facto_loop_miner_io::err::VPathUnwrapper;
    use facto_loop_miner_io::io::read_entire_file_usize_mmap_custom;
    use num_format::ToFormattedString;
    use std::path::Path;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    pub fn inner_main() {
        match 1 {
            1 => test_u8(&file_path),
            2 => io_uring_main(&file_path).expect("Asd"),
            _ => panic!("nope"),
        }
    }

    fn test_u8(path: &Path) {
        let watch = Instant::now();
        // let output = read_entire_file(path, true).unwrap();
        // let checksum = checksum_vec_u8(output);

        info!("asdf");
        let stopwatch = Instant::now();
        let output = read_entire_file_usize_mmap_custom(path, true, true, true).unwrap_path(path);
        // let output = read_entire_file_mmap_copy(path).unwrap();
        info!(
            "file read in {}",
            (Instant::now() - stopwatch)
                .as_secs()
                .to_formatted_string(&LOCALE)
        );

        // let tester = MmapOptions::new()
        //     .huge(None)
        //     .len(1024 ^ 3)
        //     .map_anon()
        //     .map_err(VIoError::io_error(path))
        //     .unwrap();
        // let checksum = checksum_vec_u8(&tester[..]);
        // info!(
        //     "tester checksum in {}",
        //     (Instant::now() - stopwatch)
        //         .as_secs()
        //         .to_formatted_string(&LOCALE)
        // );

        sleep(Duration::new(999999, 0));

        let stopwatch = Instant::now();
        let checksum = facto_loop_miner_io::checksum_vec_usize(&output) + todo!();
        info!(
            "checksum in {}",
            (Instant::now() - stopwatch)
                .as_secs()
                .to_formatted_string(&LOCALE)
        );
        println!("checksum {}", checksum);

        let time = Instant::now() - watch;
        info!(
            "InnerMain u8_unconverted {} in {}",
            path.display(),
            time.as_millis().to_formatted_string(&LOCALE)
        );
    }
}
