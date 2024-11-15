{% import "macros/fields.tera" as fields %}

# {{ edge_name }}
{{ fields::write_doc(doc_comments=doc_comments, comments=comments)}}

## Connections

| source  | target | source out | target in |
|---------|--------|--------------|--------------|
{% for endpoint in endpoints -%}
| {{ endpoint.source }}  | {{ endpoint.target }} | {{ endpoint.outgoing_bounds }} | {{ endpoint.incoming_bounds }} |
{% endfor %}

## Body
{{ fields::write_fields(fields=fields)}}

## JSON Representation
```json
{{ example }}
```