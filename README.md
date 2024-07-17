ACMG
=

A very simple CLI for figuring out what all those ACMG codes mean. Implementing the updated points-based classification
recommended by the ClinGen [Sequence Variant Interpretation (SVI)](https://clinicalgenome.org/working-groups/sequence-variant-interpretation/) group
from the paper _Fitting a naturally scaled point system to the ACMG/AMP variant classification guidelines_ - Tavtigian et al. 2020,
DOI: https://doi.org/10.1002/humu.24088

```shell
$ acmg info PVS1,PS1,PM2_Supporting
```

```
PVS1: 8 'Null variant (nonsense, frameshift, canonical ±1 or 2 splice sites, initiation codon, single or multiexon deletion) in a gene where LOF is a known mechanism of disease'
PS1 : 4 'Same amino acid change as a previously established pathogenic variant regardless of nucleotide change'
PM2_Supporting: 1 'Absent from controls (or at extremely low frequency if recessive) in Exome Sequencing Project, 1000 Genomes Project, or Exome Aggregation Consortium'
--------
Classification: Pathogenic
ACMG Score: 13
Post Prob Path: 0.999
```