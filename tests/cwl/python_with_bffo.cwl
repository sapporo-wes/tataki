# Test fixture for tataki's own integration tests. Do not copy this as an example of
# how to write a CWL document: the BFFO term below is invented. Use cwl/template.cwl as
# the template and cwl/sam_view.cwl as a working example.

cwlVersion: v1.2
class: CommandLineTool

requirements:
  DockerRequirement:
    dockerPull: python:3.11.8-alpine3.19
  InlineJavascriptRequirement: {}

baseCommand: ["python"]

successCodes: [0, 139]

inputs:
  input_file:
    type: File
    inputBinding:
      position: 1

outputs: {}

$namespaces:
  tataki: https://github.com/sapporo-wes/tataki

tataki:edam_id: http://edamontology.org/format_3996
tataki:label: Python script
# BFFO has no term for Python scripts, so this id names nothing real. A term that does
# exist would also sit in src/tataki_formats_edam_bffo.csv, and the test could then not
# tell whether the reported term came from this document or from that table. Naming a
# term the table does not hold is what makes the precedence observable.
tataki:bffo_id: https://bffo.org/format/Python/
tataki:bffo_label: Python
