# json_exporter

Prometheus requires metrics to be exposed in its own format. This exporter automatically converts a JSON endpoint to Prometheus metrics without configuration for each field containing a number or boolean field.

For instance:

```json
{
  "last_refresh_epoch": 1631046901,
  "num_http_requests": 13
}
```

gets automatically transformed into:

```
last_refresh_epoch 1631046901,
num_http_requests: 13
```

If your JSON response contains properties with values other than numbers or booleans, json_exporter ignores them by default.
It is possible, however, to leverage these properties to add additional context to a single metric.

## Usage

```bash
$ json_exporter <HTTP Endpoint serving JSON Data> -c config.yml -e '<entry_point>'
```

## Configuration

As mentioned before, JSON properties with numeric or boolean values get converted automatically. JSON responses with a more complex structure require additional configuration.

Consider this JSON:

```json
{
  "environment": "production",
  "id": "xyz",
  "last_refresh_epoch": 1631046901,
  "components": {
      "network": {
          "status": "OK",
          "status_upstream": "active",
          "has_ip_addresses": true,
          "use_ip_v6": false,
          "upstream_endpoints": 54
      },
      "router": {
          "status": "Warning",
          "num_active_uplinks": 1,
          "num_uplinks": 2
      }
  }
}
```

By default, only `last_refresh_epoch` gets transformed into a metric:

```
last_refresh_epoch 1631046901
```

### Labels

To add more context, in your yaml configuration file, please add:

```
global_labels:
  - name: environment
    selector: .environment
  - name: id
    selector: .id
```

Which then will result in metrics with labels:

```
last_refresh_epoch{environment="production",id="xyz"} 1631046901
num_requests{environment="production",id="xyz"} 42
```

`name` defines the label name, `selector` must contain a valid `jq` filter.

### Converting nested objects

It is possible to convert nested objects.

```json
{
    "environment": "production",
    "id": "xyz",
    "last_refresh_epoch": 1631046901,
    "num_requests": 42,
    "components": {
        "network": {
            "status": "OK",
            "status_upstream": "active",
            "has_ip_addresses": true,
            "use_ip_v6": false,
            "upstream_endpoints": 54
        },
        "router": {
            "status": "Warning",
            "num_active_uplinks": 1,
            "num_uplinks": 2
        }
    }
}
```

To convert `network` and `router` from the above example, your configuration file needs to look like this:

```yaml
gauge_field: status
global_labels:
  - name: environment
    selector: .environment
  - name: id
    selector: .id
gauge_field_values:
  - warning
  - critical
  - ok
```

`gauge_field` tells json_exporter which field to convert into a gauge value. Without any further configuration, if `gauge_field` contains a number, `"OK"` or `"ERROR"`, will be automatically converted.

If `gauge_field` contains any other values, please configure `gauge_field_values`. For each entry in `gauge_field_values`, you will receive one metric:

```
router_status{environment="production",id="xyz",num_active_uplinks=1,num_uplinks=2,status="warning"} 1
router_status{environment="production",id="xyz",num_active_uplinks=1,num_uplinks=2,status="critical"} 0
router_status{environment="production",id="xyz",num_active_uplinks=1,num_uplinks=2,status="ok"} 0

network_status{environment="production",id="xyz",status_upstream="active",has_ip_addresses=true,use_ip_v6=false,upstream_endpoints=54,status="warning"} 0
network_status{environment="production",id="xyz",status_upstream="active",has_ip_addresses=true,use_ip_v6=false,upstream_endpoints=54,status="critical"} 0
network_status{environment="production",id="xyz",status_upstream="active",has_ip_addresses=true,use_ip_v6=false,upstream_endpoints=54,status="ok"} 1
```

If the values are nested, you need to provide an entry point in `jq` notation:

```
$json_exporter http://localhost:8800/json -c config.yaml -e ".components"
```

### Custom Includes

If you'd like to include metrics for JSON objects that are nested and wouldn't otherwise be generated automatically, it's possible to configure json exporter to also fetch and convert those.

In your config file, please add:

```json
includes:
    - name: router_backend_status
      label_name: backend
      label_selector: .router.backend
      selector:
        - ".router.backend.back1"
        - ".router.backend.back2"
```

**Explanation**:

`name`: The resulting metric name
`label_name`: Since you can provide multiple selectors, each resulting metric will have an attribute indicating from which JSON Object it originated. `label_name` allows you to configure the name for that label.
`label_selector`: A valid `jq` selector to fetch the value for above-mentioned label.
`selector`: One or more valid `jq` selectors that specify paths for JSON objects to retrieve

Example:

## Development

- >= Rust 1.54
- `brew install jq`

## Production

System requirements:

- `jq` needs to be present in `$PATH`

## License

MIT