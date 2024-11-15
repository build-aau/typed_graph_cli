{% import "macros/fields.tera" as fields %}

# {{ node_name }}
{{ fields::write_doc(doc_comments=doc_comments, comments=comments)}}

## Connections

### Outgoing
| edge | source | target | bounds |
|------|--------|--------|--------|
{% for endpoint in outgoing_endpoints -%}
| {{ endpoint.edge }} | {{ endpoint.source }}  | {{ endpoint.target }} | {{ endpoint.bounds}} |
{% endfor %}

### Incoming
| edge | source | target | bounds |
|------|--------|--------|--------|
{% for endpoint in incoming_endpoints -%}
| {{ endpoint.edge }} | {{ endpoint.source }}  | {{ endpoint.target }} | {{ endpoint.bounds}} |
{% endfor %}

## Body
{{ fields::write_fields(fields=fields)}}

## JSON Representation
```json
{{ example }}
```