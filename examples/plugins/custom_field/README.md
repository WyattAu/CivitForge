# Custom Field Plugin

Adds custom fields to issues with type validation and persistence.

## Data Model

Custom fields are stored as key-value pairs in the plugin's persistent storage, keyed by issue ID and field name.

### Storage Format

```
Key:   custom_field:<issue_id>:<field_name>
Value: CustomField JSON
```

### CustomField struct

```json
{
  "name": "priority",
  "field_type": "select",
  "value": "high"
}
```

### Supported Field Types

| Type | Validation |
|------|-----------|
| `text` | Must be a string |
| `number` | Must be a number |
| `boolean` | Must be a boolean |
| `select` | Must be one of `allowed_values` |

## Configuration

```json
{
  "field_name": "priority",
  "field_type": "select",
  "required": true,
  "default_value": "medium",
  "allowed_values": ["low", "medium", "high", "critical"]
}
```

| Key | Type | Required | Description |
|-----|------|----------|-------------|
| `field_name` | string | yes | Name of the custom field |
| `field_type` | string | yes | One of: `text`, `number`, `boolean`, `select` |
| `required` | boolean | yes | Whether the field must be present |
| `default_value` | string | no | Default if not provided |
| `allowed_values` | string[] | no | Required for `select` type |

## Build

```bash
cargo component build --release
```

## Install

```bash
cp target/wasm32-wasip1/release/custom_field_plugin.wasm ~/.civitforge/plugins/
```

## Test

```bash
cargo test
```
