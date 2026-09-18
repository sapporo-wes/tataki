cwlVersion: v1.2
class: CommandLineTool

requirements:
  DockerRequirement:
    dockerPull: your_docker_image
  InlineJavascriptRequirement: {}

baseCommand: [command, to, use]

successCodes: [0, 139]

inputs:
  input_file:
    type: File
    inputBinding:
      position: 1

outputs: {}

$namespaces:
  tataki: https://github.com/sapporo-wes/tataki
  
tataki:edam_id: http://edamontology.org/format_edam-id
tataki:label: edam-label

# Optional. Omit to let tataki look the BFFO term up from the edam_id above.
# tataki:bffo_id: https://bffo.org/format/bffo-slug/
# tataki:bffo_label: bffo-slug