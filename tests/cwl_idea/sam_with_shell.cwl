cwlVersion: v1.2
class: CommandLineTool

requirements:
  DockerRequirement:
    dockerPull: quay.io/biocontainers/samtools:1.18--h50ea8bc_1
  InlineJavascriptRequirement: {}

baseCommand: [bash, -c]

arguments:
  - valueFrom: |
      samtools head $(inputs.input_file.path) > /dev/null 2>&1
      echo $? > exit_code.txt
    shellQuote: false

inputs:
  input_file:
    type: File

outputs:
  result:
    type: boolean
    outputBinding:
      glob: exit_code.txt
      loadContents: true
      outputEval: |
        ${
          return self[0].contents.trim() === "0";
        }
