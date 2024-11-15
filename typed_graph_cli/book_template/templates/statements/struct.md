{% import "macros/fields.tera" as fields %}

# {{ struct_name }}
{{ fields::write_doc(doc_comments=doc_comments, comments=comments)}}

## Body
{{ fields::write_fields(fields=fields)}}

## JSON Representation
```json
{{ example }}
```