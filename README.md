# json_exporter

Prometheus requires metrics to be exposed in its own format. This exporter automatically converts a JSON endpoint to Prometheus metrics.

## Usage

```bash
$ json_exporter <HTTP Endpoint serving JSON Data>
```

## Yaml Overrides

Consider this JSON:

```json
{"status":"success","code":0,"data":{"UserCount":140,"UserCountActive":23}}
```

It automatically gets transformed to:

```
data{user_count=140,user_count_active=23}
```

If you'd like to change the metrics to:

```
data_user_count 140
data_user_count_active 23
```

Create a new `overrides.yml`:

```yaml
metrics:
  - name: data_user_count
    selector: "{.data.UserCount}"
  - name: data_user_count_active
    selector: "{.data.UserCountActive}"
```

and when invoking `json_exporter`, provide it as argument:

```bash
$ json_exporter -c overrides.yml <HTTP Endpoint serving JSON Data>
```

## Development

- >= Rust 1.54