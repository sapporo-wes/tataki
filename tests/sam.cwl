cwlVersion: v1.2
class: CommandLineTool

requirements:
  DockerRequirement:
    dockerPull: quay.io/biocontainers/samtools:1.18--h50ea8bc_1
  InlineJavascriptRequirement: {}

baseCommand: [samtools, head]

inputs:
  input_file:
    type: File
    inputBinding:
      position: 1

outputs:
  result:
    type: boolean
    outputBinding:
      outputEval: |
        ${
          return runtime.exitCode === 0;
        }
