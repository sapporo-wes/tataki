cwlVersion: v1.2
class: CommandLineTool

requirements:
  DockerRequirement:
    dockerPull: quay.io/biocontainers/samtools:1.18--h50ea8bc_1
  InlineJavascriptRequirement: {}

baseCommand: [samtools, head]

successCodes: [0, 139]

inputs:
  input_file:
    type: File
    inputBinding:
      position: 1

outputs:
  label:
    type: string
    outputBinding:
      outputEval: '${ return "sam" }'
  edam_id:
    type: string
    outputBinding:
      outputEval: '${ return "format_2573" }'
  result:
    type: boolean
    outputBinding:
      outputEval: '${ return runtime.exitCode === 0 }'
