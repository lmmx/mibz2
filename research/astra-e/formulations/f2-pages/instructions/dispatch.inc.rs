    fn build(&mut self, quote: u32, tokens: u32) {
        #[cfg(any(target_arch="x86", target_arch="x86_64"))]
        {
            if MODE == 3 && std::is_x86_feature_detected!("popcnt") && std::is_x86_feature_detected!("bmi2") {
                // SAFETY: both features required by this routine were detected on this CPU.
                return unsafe { self.build_bmi(quote, tokens) };
            }
            if MODE != 0 && MODE != 4 && std::is_x86_feature_detected!("popcnt") {
                // SAFETY: the hardware population count feature has been detected.
                return unsafe { self.build_pop(quote, tokens) };
            }
        }
        self.build_body(quote, tokens)
    }

    fn prepare_counted(&mut self, caps: &[u16], n: u32, counts: &[u32;256]) -> Option<(u32,u32)> {
        #[cfg(any(target_arch="x86", target_arch="x86_64"))]
        if MODE >= 2 && std::is_x86_feature_detected!("popcnt") {
            // SAFETY: only the detected hardware population count feature is enabled.
            return unsafe { self.prepare_pop(caps,n,counts) };
        }
        self.prepare_body(caps,n,counts)
    }

    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    #[target_feature(enable="popcnt")]
    unsafe fn prepare_pop(&mut self, caps: &[u16], n: u32, counts: &[u32;256]) -> Option<(u32,u32)> {
        self.prepare_body(caps,n,counts)
    }

    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    #[target_feature(enable="popcnt")]
    unsafe fn build_pop(&mut self, quote:u32, tokens:u32) {
        self.build_body(quote,tokens)
    }

    #[cfg(any(target_arch="x86", target_arch="x86_64"))]
    #[target_feature(enable="popcnt,bmi2")]
    unsafe fn build_bmi(&mut self, quote:u32, tokens:u32) {
        self.build_body(quote,tokens)
    }
