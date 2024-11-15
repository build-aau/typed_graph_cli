{% import "macros/fields.tera" as fields %}

# {{ enum_name }}
{{ fields::write_doc(doc_comments=doc_comments, comments=comments)}}

## Varients
{% for varient in varients %}

{% if varient.varient_type == "Unit" %}
#### {{ varient.name }}  
{{ fields::write_doc(doc_comments=varient.doc_comments, comments=varient.comments)}}
## JSON Representation
```json
{{ varient.example }}
```
{% elif varient.varient_type == "Struct"%}
#### {{ varient.name }}  
## Body
{{ fields::write_doc(doc_comments=varient.doc_comments, comments=varient.comments)}}
{{ fields::write_fields(fields=varient.root_fields)}}
## JSON Representation
```json
{{ varient.example }}
```

{% elif varient.varient_type == "Opaque"%}
#### {{ varient.name }} ({{ varient.root_ty }})  
{{ fields::write_doc(doc_comments=varient.doc_comments, comments=varient.comments)}}
## JSON Representation
```json
{{ varient.example }}
```

{% else %}
#### {{ varient.name }}  
{{ fields::write_doc(doc_comments=varient.doc_comments, comments=varient.comments)}}

{% endif%}
{% endfor %}
