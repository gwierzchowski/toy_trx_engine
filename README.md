TODO: It looks like error handling in case when `trim` on CSV reader is not set and there are spaces between data is not good (rec, line, byte info is misleading), and also in case when `has_headers` is not propoerly set. To investigate and possibly open issue/PR for `csv-core` crate 

TODO: Float calculations.

TODO: Logging, 

TODO: Printing output file: use more robust printing interface for accounts (Display, print to Writer etc.) instead of print!.

TODO: Memory allocation optimization - limit number of allocations per transaction (possibly to 1 or none).

TODO: `data: AsyncDeserializer<R>` argument of `processing_loop` is not ideal - should be more abstract stream-like data type.

TODO: `csv_async` should re-export futures/tokio::io::AsyncRead.
