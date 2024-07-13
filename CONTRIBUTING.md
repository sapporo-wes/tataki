# Contrubuting Guidelines

Thank you for considering contributing to [tataki](https://github.com/sapporo-wes/tataki)! Here are some guidelines to help you get started.

## Contributing to the modules

### I want to add a module to the standalone mode


### I want to add a CWL document to the external extension mode

1. Create a CWL document under the [`cwl`](cwl/) directory.
2. Create a pull request.

Please make sure that your CWL document has the following:

- `edam_id` and `label`: Both must have `tataki` prefix which is listed in the `$namespaces` section.

Example of a CWL document:

```cwl
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

outputs: {}

$namespaces:
  tataki: https://github.com/sapporo-wes/tataki
  
tataki:edam_id: http://edamontology.org/format_2573
tataki:label: SAM
```

