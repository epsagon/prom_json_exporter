# json_exporter

Prometheus requires metrics to be exposed in its own format. This exporter automatically converts a JSON endpoint to Prometheus metrics.

## Usage

```bash
$ json_exporter <HTTP Endpoint serving JSON Data>
```

## Development

- >= Rust 1.54