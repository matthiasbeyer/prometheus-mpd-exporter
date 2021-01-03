# prometheus-mpd-exporter

Export MPD server metrics to prometheus.


## Usage

```
RUST_LOG=info \
prometheus-mpd-exporter                            \
    --mpd-server-addr 127.0.0.1                    \
    --mpd-server-port 6600                         \
    --bind-addr <address to bind to, e.g. 0.0.0.0> \
    --bind-port <port to bind to, e.g. 9123>
```

And then start scraping with prometheus.


## License

GPLv2.0 only

