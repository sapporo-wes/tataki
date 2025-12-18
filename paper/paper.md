---
title: 'Tataki: Enhancing the robustness of bioinformatics workflows with simple, tolerant file format detection'
tags:
  - Bioinformatics
  - Computational genomics
  - Workflow
  - Common Workflow Language
  - File format detection
authors:
  - name: Masaki Fukui
    given-names: Masaki
    surname: Fukui
    orcid: 0009-0001-3678-3006
    affiliation: "1"
    email: fukui@sator.co.jp
  - name: Hirotaka Suetake
    given-names: Hirotaka
    surname: Suetake
    orcid: 0000-0003-2765-0049
    affiliation: "1"
    email: suecharo@sator.co.jp
  - name: Tazro Ohta
    given-names: Tazro
    surname: Ohta
    orcid: 0000-0003-3777-5945
    affiliation: "2, 3"
    email: tazro.ohta@chiba-u.jp
affiliations:
 - name: Sator, Inc.
   index: 1
 - name: Institute for Advanced Academic Research, Chiba University
   index: 2
   ror: 01hjzeq58
 - name: Department of Artificial Intelligence Medicine, Graduate School of Medicine, Chiba University
   index: 3
date: XX December 2025
bibliography: paper.bib
---

# Summary

`Tataki` is a lightweight command-line tool for identifying the file formats used in bioinformatics workflows. Modern genomic analyses rely on many intermediate files - such as sequence alignments, variant calls, and genomic feature annotations - to connect multiple tools in automated pipelines. However, these files are often malformed, truncated, mislabeled, or inconsistent with their file extensions, and many tools do not reliably detect such issues. As a result, workflow failures can occur silently and are often difficult to diagnose.

`Tataki` addresses this problem by examining the actual contents of a file and determining its format using strict, domain aware parsers. It can detect empty files, mixed content files, or subtle format inconsistencies that generic file identification tools commonly miss. `Tataki` currently supports widely used genomics formats and provides an extensible mechanism - based on the Common Workflow Language (CWL) - that allows users to customize or extend format identification rules for project specific needs.

By inserting `Tataki` between workflow steps, researchers can detect anomalies early, prevent error propagation, and improve the robustness and reproducibility of automated analyses. `Tataki` is designed to be simple, composable, and workflow-friendly, making it suitable for integration into a wide range of bioinformatics pipelines.

# Statement of need

Modern bioinformatics workflows integrate many specialized analytical tools to process large scale sequencing data [@perkel_workflow_2019]. These workflows are designed to reduce manual intervention and to ensure reproducibility by automating complex multi-step analyses. However, despite their widespread adoption, workflow executions often fail for surprisingly simple reasons: intermediate files may be empty, truncated, or internally inconsistent, and many bioinformatics tools do not reliably signal such failures through exit codes [@niu_assessing_2022]. As a consequence, downstream tools may misinterpret file formats, propagate incorrect assumptions, or terminate unexpectedly. These silent errors reduce the overall robustness and fault tolerance of automated workflows, which becomes increasingly problematic as datasets grow and manual oversight becomes infeasible.

This fragility is rooted in longstanding characteristics of the bioinformatics ecosystem. The field contains a large and heterogeneous collection of file formats, many of which lack formal specifications or have multiple variants used by different tools. Beyond a handful of well-standardized formats such as SAM/BAM [@li_sequence_2009]/CRAM [@cochrane_facing_2012], VCF [@danecek_variant_2011] or BED [@niu_assessing_2022], many commonly used formats have ambiguous boundaries or are interpreted differently across implementations [@rehm_ga4gh_2021]. At the same time, bioinformatics software is often developed by individual research groups for specific tasks, typically originating as stand-alone research code created under tight time and resource constraints, leading to inconsistent behaviors, uneven error handling, and non-standard assumptions about input data [@brack_ten_2022]. These factors combine to create a landscape in which workflows must routinely exchange files whose correctness cannot be assumed.

Existing approaches only partially address this issue. Generic file identification utilities (e.g., `file` [@noauthor_fine_nodate], or AI-powered detectors such as Magika [@fratantonio_magika_2025]) are not designed to understand domain-specific bioinformatics formats and often cannot distinguish superficially valid but malformed files. Research efforts such as conformance testing for BED parsers have demonstrated the value of rigorous validation for individual formats [@niu_assessing_2022], but such approaches do not scale to the wide variety of formats encountered in real workflows. More importantly, these tools are rarely designed as small, composable components that can be seamlessly inserted between workflow steps.

To address these challenges, we developed `Tataki`, a lightweight and workflow-friendly file format detection tool tailored to bioinformatics. `Tataki` inspects actual file content using strict parsers rather than relying on heuristics, allowing it to detect empty, truncated, mixed, or otherwise anomalous files. It currently supports major genomics formats and performs strict validation of their expected structure. Flexibility for handling format variants that arise in practical workflows is provided through its External Extension Mode, which integrates with the CWL [@crusoe_methods_2022]. This mode enables users to extend or customize format identification rules, including those for emerging or project-specific formats, without modifying the core software.

By incorporating `Tataki` between workflow steps, researchers and developers can rapidly identify format-related anomalies before they propagate, improve debugging of multi-step pipelines, and record more accurate provenance for workflow outputs. `Tataki` is particularly useful in large scale automated analyses where silent file corruption or tool misbehavior can otherwise consume substantial compute resources and obscure the root cause of workflow failures. As a small, standalone command-line utility, `Tataki` supports integration into diverse workflow systems, including CWL-based engines and other task orchestration frameworks.

In summary, `Tataki` addresses a fundamental and widely encountered limitation in bioinformatics workflows: the lack of reliable, composable, domain-aware file format checks. By providing a simple mechanism to validate the integrity and identity of files at each stage of the workflow, this approach improves workflow robustness and supports reproducible research across a range of computational genomics applications.

# External Extension Mode

In addition to its built-in format detectors, `Tataki` provides an External Extension Mode that enables users to define custom file format identification logic through the CWL. This mode is intended for situations in which researchers must support formats not included in `Tataki`'s native detectors or wish to validate files using existing command-line tools.

External Extension Mode delegate format recognition to a user-supplied CWL document, which specifies how a file should be processed and which EDAM [@Black2022] format identifier should be assigned upon successful validation. This design allows researchers to incorporate their own validators without modifying `Tataki`'s core software, while still maintaining clear semantic definitions through the EDAM ontology.

By supporting user-defined format detection through CWL, External Extension Mode allows `Tataki` to remain lightweight while accommodating the diverse and evolving landscape of file formats used in bioinformatics research. This approach provides a fleible pathway for integrating domain-specific validators and facilitates more reliable workflow execution in specialized or rapidly developing research areas.

# Limitations

While `Tataki` improves robustness in bioinformatics workflows by validating file formats at workflow boundaries, it has several limitations. First, the tool currently supports only a selected set of widely used genomics file types. Although the External Extension Mode allows users to define additional detectors through CWL, creating and maintaining these definitions requires familiarity with CWL and may introduce overhead for users who are not already using CWL.

Second, `Tataki`'s format identification relies on strict parsers to detect malformed or inconsistent contents. While this approach improves reliability, it may reject files that are technically valid but contain format variations tolerated by some downstream tools. As a result, users may need to adjust their workflow or extend `Tataki` with custom detectors to accommodate project-specific conventions.

Finally, `Tataki` focuses on identifying file formats and detecting structural anomalies; it does not perform semantic validation of biological content. For example, it does not verify whether sequence identifiers are consistent across related files or whether genomic coordinates fall within valid ranges. Such checks remain the responsibility of downstream analysis tools or dedicated validation software.

# Acknowledgements

This work was supported by the research grant from the SECOM Science and Technology Foundation (FY2023 grant program).

# References

