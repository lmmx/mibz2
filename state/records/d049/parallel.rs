                });
                if l.rle_panicked {
                    t.bufs[1 - a] = Vec::new();
                }
            }
            Ok(Err(p)) => {
                // a helper's stage panicked: block i fails, and the back set is reset
                let kind = ErrorKind::Internal(panic_message(p));
                publish(shared, s.place, s.bit, Err(kind), s.symbols.syms);
                t.tt = Default::default();
                t.seg = Default::default();
                t.bufs = Default::default();
            }
            Err(p) => {
                // the run itself panicked (the leader's part catches its own stages', so a
                // bug outside them): every block the run held fails, and the team is reset
                let msg = panic_message(p);
                let held = [Some(s), l.next.take()];
                for b in held.into_iter().flatten() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                if let Some(b) = l.inverted.take() {
                    let kind = ErrorKind::Internal(msg.clone());
                    publish(shared, b.place, b.bit, Err(kind), b.symbols.syms);
                }
                t = Team::default();
