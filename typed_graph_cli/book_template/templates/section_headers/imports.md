# Imports
## NodeId and EdgeId
Type used to identify nodes and edges in the grpah.

Commonly this is set to UUID or int but for more specialized use cases other type may be used.

**JSON Representation**  
UUID = `"30c0d1e5-f9aa-4b26-90b9-0a01f6f0ae25"`  
int = `123456`

{%- for import in imports %}
## {{ import.name }}
{{ import.doc_comments }}
{% if import.comments %}
**Extra comments:**  
{{ import.comments}}
{% endif %}
{% endfor %}