        "parity" => {
            let mut kr = crate::reference::Kernel::default();
            let mut k = Kernel::default();
            let (mut master, mut actual) = (Vec::new(), Vec::new());
            for b in &bs {
                decode_ref(&mut kr, b, &mut master);
                let stats = decode(&mut k, b, &mut actual);
                assert!(actual == master && actual == crate::open::b5::plain(&b.1, b.2), "{}", b.0);
                println!("MEM {} {} {}", b.0, b.1.len(), stats.retained);
            }
        }
        "corpus" => {
            let data: &'static [u8] = Box::leak(std::fs::read(std::env::var("WIKIDATA").unwrap()).unwrap().into_boxed_slice());
            let workers = std::env::var("WORKERS").unwrap().parse().unwrap();
            let reverse: usize = std::env::var("REVERSE").unwrap().parse().unwrap();
            let mut times = [0.; 2];
            #[cfg(feature = "block-clocks")]
            let mut blocks = [[0.; 431]; 2];
            for i in 0..2 {
                let arm = (reverse + i) % 2;
                times[arm] = if arm == 0 {
                    crate::corpuscheck::stream::<crate::reference::OpenCells>(data, workers)
                } else {
                    crate::corpuscheck::stream::<crate::ARM::OpenCells>(data, workers)
                };
                #[cfg(feature = "block-clocks")]
                { blocks[arm] = crate::corpuscheck::read_times(); }
            }
            #[cfg(not(feature = "block-clocks"))]
            println!("CORP stream.0 {:.3} {:.3}", times[0], times[1]);
            #[cfg(feature = "block-clocks")]
            for id in 0..431 {
                println!("T1 blk{id:03} {:.3} {:.3}", blocks[0][id], blocks[1][id]);
            }
        }
