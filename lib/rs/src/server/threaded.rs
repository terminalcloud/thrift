/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements. See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership. The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License. You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied. See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

use std::sync::{Arc, mpsc};
use std::thread;

use transport::server::TransportServer;
use transport::Transport;
use protocol::{Protocol, ProtocolFactory};
use processor::Processor;

pub struct ThreadedServer<P, PF, TS> {
    inner: Arc<ThreadedServerInner<P, PF, TS>>
}

struct ThreadedServerInner<P, PF, TS> {
    processor: P,
    protocol_factory: PF,
    transport_server: TS
}

impl<P, PF, TS> ThreadedServer<P, PF, TS>
where P: Processor<PF::Protocol, TS::Transport> + Send + Sync + 'static,
      TS: TransportServer + Send + Sync + 'static,
      PF: ProtocolFactory + Send + Sync + 'static,
      // FIXME(reem): This bound is redundant but is needed to get around an ICE
      // in 1.0 stable.
      TS::Transport: Transport {

    pub fn new(processor: P, factory: PF, server: TS) -> Self {
        ThreadedServer {
            inner: Arc::new(ThreadedServerInner {
                processor: processor,
                protocol_factory: factory,
                transport_server: server
            })
        }
    }

    pub fn serve(self, threads: usize) {
        assert!(threads != 0, "Can't accept on 0 threads.");

        let (supervisor_tx, supervisor_rx) = mpsc::channel();

        for _ in (0..threads) {
            self.spawn_with_supervisor(supervisor_tx.clone());
        }

        // Instead of holding on to this for future calls to
        // spawn_with_supervisor, we drop it here so the only
        // sending handles are those in worker threads.
        //
        // This means the loop over supervisor_rx will terminate
        // when all worker threads have completed succesfully.
        drop(supervisor_tx);

        for PanicMessage(supervisor_tx) in supervisor_rx.iter() {
            self.spawn_with_supervisor(supervisor_tx.clone());
        }
    }

    fn spawn_with_supervisor(&self, supervisor: mpsc::Sender<PanicMessage>) {
        let shared = self.inner.clone();

        thread::spawn(move || {
            let _sentinel =
                Sentinel::new(supervisor.clone(), PanicMessage(supervisor));

            loop {
                let mut transport = shared.transport_server.accept().unwrap();
                let mut protocol = shared.protocol_factory.new_protocol();

                while let Ok(_) =
                    shared.processor.process(&mut protocol, &mut transport) { }
            }
        });
    }
}

struct PanicMessage(mpsc::Sender<PanicMessage>);

struct Sentinel<T: Send + 'static> {
    value: Option<T>,
    supervisor: mpsc::Sender<T>,
}

impl<T: Send + 'static> Sentinel<T> {
    fn new(channel: mpsc::Sender<T>, data: T) -> Sentinel<T> {
        Sentinel {
            value: Some(data),
            supervisor: channel,
        }
    }
}

impl<T: Send + 'static> Drop for Sentinel<T> {
    fn drop(&mut self) {
        // Ignore failure of the supervisor thread so as to avoid double panics
        // due to supervisor failure followed by child failure.
        let _ = self.supervisor.send(self.value.take().unwrap());
    }
}

