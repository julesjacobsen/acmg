use std::cmp::PartialEq;
use std::collections::{BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use regex::Regex;

use crate::Category::{Benign, Pathogenic};
use crate::EvidenceStrength::{Moderate, StandAlone, Strong, Supporting, VeryStrong};

#[derive(Parser)]
#[command(name = "acmg", version = "0.1.0")]
#[command(bin_name = "acmg")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Shows info about ACMG codes
    #[command(arg_required_else_help = true,
        name = "info",
        about = "Calculates ACMG score and classifies pathogenicity from ACMG evidence codes",
    )]
    Info {
        /// ACMG evidence string, e.g 'PVS1, PM2_Supporting'
        acmg_evidence: String,
    },
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Info { acmg_evidence } => {
            run_info_command(&acmg_evidence);
        }
    }
}

fn run_info_command(acmg_evidence: &str) {
    let evidence_list = normalize_input(&acmg_evidence);
    let set = BTreeSet::from_iter(evidence_list.iter()
        .map(|evidence_code| parse_evidence(evidence_code).unwrap()));
    let mut score = 0;
    for evidence in set {
        let evidence_code = evidence.evidence_code;
        let points = evidence.points();
        println!("{:4}:{:2} '{}'", evidence, points, evidence_code.description);
        score += points;
    }
    println!("--------");
    println!("Classification: {:?}", classification(score));
    println!("ACMG Score: {}", score);
    println!("Post Prob Path: {:.3}", calc_post_prob_path(score));
}

fn normalize_input(acmg_evidence: &str) -> Vec<String> {
    let re = Regex::new(r"[\[\]]").unwrap();
    let cleaned = re.replace_all(acmg_evidence, "").trim().to_string();
    Regex::new(r"[ ,]+").unwrap().split(&cleaned).map(|s| s.to_string()).collect()
}

fn parse_evidence(evidence: &str) -> Result<Evidence, String> {
    if let Some(caps) = RE.captures(&evidence.to_uppercase()) {
        let ev_code_str = caps.get(1).map_or("", |m| m.as_str());
        let evidence_code = match EVIDENCE_CODES.get(ev_code_str) {
            Some(ev) => ev,
            None => return Err(format!("Invalid evidence code {}", ev_code_str)),
        };

        let modifier = match caps.get(3).map_or("", |m| m.as_str()).to_uppercase().as_str() {
            "STANDALONE" => Option::from(StandAlone),
            "VERYSTRONG" => Option::from(VeryStrong),
            "STRONG" => Option::from(Strong),
            "MODERATE" => Option::from(Moderate),
            "SUPPORTING" => Option::from(Supporting),
            "" => None,
            s => return Err(format!("Invalid modifier '{}' for evidence code {}", s, evidence)),
        };
        return Ok(Evidence { evidence_code, modifier });
    }
    Err(format!("Unable to parse evidence code {}", evidence))
}

fn calc_post_prob_path(points: i32) -> f64 {
    let odds_path = ODDS_PATH_SUPPORTING.powi(points);
    (odds_path * PRIOR_PROB) / ((odds_path - 1.0) * PRIOR_PROB + 1.0)
}

fn classification(points: i32) -> AcmgClassification {
    match points {
        p if p >= 10 => AcmgClassification::Pathogenic,
        p if p >= 6 => AcmgClassification::LikelyPathogenic,
        p if p >= 0 => AcmgClassification::UncertainSignificance,
        p if p >= -6 => AcmgClassification::LikelyBenign,
        _ => AcmgClassification::Benign,
    }
}

#[derive(Debug)]
enum AcmgClassification {
    Pathogenic,
    LikelyPathogenic,
    UncertainSignificance,
    LikelyBenign,
    Benign,
}

#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
enum Category {
    Pathogenic,
    Benign,
}

#[derive(Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
enum EvidenceStrength {
    StandAlone,
    VeryStrong,
    Strong,
    Moderate,
    Supporting,
}

impl EvidenceStrength {
    fn points(&self) -> i32 {
        match self {
            StandAlone | VeryStrong => 8,
            Strong => 4,
            Moderate => 2,
            Supporting => 1,
        }
    }
}

impl FromStr for EvidenceStrength {
    type Err = String;

    fn from_str(s: &str) -> Result<EvidenceStrength, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" | "STANDALONE" => Ok(StandAlone),
            "VS" | "VERYSTRONG" => Ok(VeryStrong),
            "S" | "STRONG" => Ok(Strong),
            "M" | "MODERATE" => Ok(Moderate),
            "P" | "SUPPORTING" => Ok(Supporting),
            _ => Err(format!("Invalid strength value: {}", s)),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct EvidenceCode {
    category: Category,
    strength: EvidenceStrength,
    code: i32,
    description: &'static str,
}

impl Display for EvidenceCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let category = match self.category {
            Pathogenic => "P",
            Benign => "B",
        };
        let strength = match self.strength {
            StandAlone => "A",
            VeryStrong => "VS",
            Strong => "S",
            Moderate => "M",
            Supporting => "P",
        };
        f.pad(&format!("{}{}{}", category, strength, self.code))
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Evidence {
    evidence_code: &'static EvidenceCode,
    modifier: Option<EvidenceStrength>,
}

impl Display for Evidence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.modifier {
            Some(modifier) => f.pad(&format!("{}_{:?}", self.evidence_code, modifier)),
            None => f.pad(&format!("{}", self.evidence_code)),
        }
    }
}

impl Evidence {
    fn points(&self) -> i32 {
        let points = self.modifier.as_ref().unwrap_or(&self.evidence_code.strength).points();
        if self.evidence_code.category == Pathogenic { points } else { -points }
    }
}

const PRIOR_PROB: f64 = 0.1;
const ODDS_PATH_VERY_STRONG: f64 = 350.0;
const EXPONENTIAL_PROGRESSION: f64 = 2.0;
lazy_static! {
    static ref RE: Regex = Regex::new(r"([BP][AVSMP]{1,2}\d{1})(_([A-Z]+))?").unwrap();
    static ref SUPPORTING_EVIDENCE_EXPONENT: f64 = EXPONENTIAL_PROGRESSION.powf(-3.0); // 0.125
    static ref ODDS_PATH_SUPPORTING: f64 = ODDS_PATH_VERY_STRONG.powf(*SUPPORTING_EVIDENCE_EXPONENT); // 2.08
    static ref EVIDENCE_CODES: HashMap<&'static str, EvidenceCode> = HashMap::from([
        // Path VeryStrong
        ("PVS1", EvidenceCode{category: Pathogenic, strength: VeryStrong, code: 1, description: "Null variant (nonsense, frameshift, canonical ±1 or 2 splice sites, initiation codon, single or multiexon deletion) in a gene where LOF is a known mechanism of disease"}),
        // Path Strong
        ("PS1", EvidenceCode{category: Pathogenic, strength: Strong, code: 1, description: "Same amino acid change as a previously established pathogenic variant regardless of nucleotide change"}),
        ("PS2", EvidenceCode{category: Pathogenic, strength: Strong, code: 2, description: "De novo (both maternity and paternity confirmed) in a patient with the disease and no family history"}),
        ("PS3", EvidenceCode{category: Pathogenic, strength: Strong, code: 3, description: "Well-established in vitro or in vivo functional studies supportive of a damaging effect on the gene or gene product"}),
        ("PS4", EvidenceCode{category: Pathogenic, strength: Strong, code: 4, description: "The prevalence of the variant in affected individuals is significantly increased compared with the prevalence in controls"}),
        // Path Moderate
        ("PM1", EvidenceCode{category: Pathogenic, strength: Moderate, code: 1, description: "Located in a mutational hot spot and/or critical and well-established functional domain (e.g., active site of an enzyme) without benign variation"}),
        ("PM2", EvidenceCode{category: Pathogenic, strength: Moderate, code: 2, description: "Absent from controls (or at extremely low frequency if recessive) in Exome Sequencing Project, 1000 Genomes Project, or Exome Aggregation Consortium"}),
        ("PM3", EvidenceCode{category: Pathogenic, strength: Moderate, code: 3, description: "For recessive disorders, detected in trans with a pathogenic variant"}),
        ("PM4", EvidenceCode{category: Pathogenic, strength: Moderate, code: 4, description: "Protein length changes as a result of in-frame deletions/insertions in a nonrepeat region or stop-loss variants"}),
        ("PM5", EvidenceCode{category: Pathogenic, strength: Moderate, code: 5, description: "Novel missense change at an amino acid residue where a different missense change determined to be pathogenic has been seen before"}),
        ("PM6", EvidenceCode{category: Pathogenic, strength: Moderate, code: 6, description: "Assumed de novo, but without confirmation of paternity and maternity"}),
        // Path Supporting
        ("PP1", EvidenceCode{category: Pathogenic, strength: Supporting, code: 1, description: "Cosegregation with disease in multiple affected family members in a gene definitively known to cause the disease"}),
        ("PP2", EvidenceCode{category: Pathogenic, strength: Supporting, code: 2, description: "Missense variant in a gene that has a low rate of benign missense variation and in which missense variants are a common mechanism of disease"}),
        ("PP3", EvidenceCode{category: Pathogenic, strength: Supporting, code: 3, description: "Multiple lines of computational evidence support a deleterious effect on the gene or gene product (conservation, evolutionary, splicing impact, etc.)"}),
        ("PP4", EvidenceCode{category: Pathogenic, strength: Supporting, code: 4, description: "Patient’s phenotype or family history is highly specific for a disease with a single genetic etiology"}),
        ("PP5", EvidenceCode{category: Pathogenic, strength: Supporting, code: 5, description: "Reputable source recently reports variant as pathogenic, but the evidence is not available to the laboratory to perform an independent evaluation"}),
        // BENIGN - Table 4 of https://www.acmg.net/docs/Standards_Guidelines_for_the_Interpretation_of_Sequence_Variants.pdf
        // Benign StandAlone
        ("BA1", EvidenceCode{category: Benign, strength: StandAlone, code: 1, description: "Allele frequency is >5% in Exome Sequencing Project, 1000 Genomes Project, or Exome Aggregation Consortium"}),
        // Benign Strong
        ("BS1", EvidenceCode{category: Benign, strength: Strong, code: 1, description: "Allele frequency is greater than expected for disorder"}),
        ("BS2", EvidenceCode{category: Benign, strength: Strong, code: 2, description: "Observed in a healthy adult individual for a recessive (homozygous), dominant (heterozygous), or X-linked (hemizygous) disorder, with full penetrance expected at an early age"}),
        ("BS3", EvidenceCode{category: Benign, strength: Strong, code: 3, description: "Well-established in vitro or in vivo functional studies show no damaging effect on protein function or splicing"}),
        ("BS4", EvidenceCode{category: Benign, strength: Strong, code: 4, description: "Lack of segregation in affected members of a family"}),
        // Benign Supporting
        ("BP1", EvidenceCode{category: Benign, strength: Supporting, code: 1, description: "Missense variant in a gene for which primarily truncating variants are known to cause disease"}),
        ("BP2", EvidenceCode{category: Benign, strength: Supporting, code: 2, description: "Observed in trans with a pathogenic variant for a fully penetrant dominant gene/disorder or observed in cis with a pathogenic variant in any inheritance pattern"}),
        ("BP3", EvidenceCode{category: Benign, strength: Supporting, code: 3, description: "In-frame deletions/insertions in a repetitive region without a known function"}),
        ("BP4", EvidenceCode{category: Benign, strength: Supporting, code: 4, description: "Multiple lines of computational evidence suggest no impact on gene or gene product (conservation, evolutionary, splicing impact, etc.)"}),
        ("BP5", EvidenceCode{category: Benign, strength: Supporting, code: 5, description: "Variant found in a case with an alternate molecular basis for disease"}),
        ("BP6", EvidenceCode{category: Benign, strength: Supporting, code: 6, description: "Reputable source recently reports variant as benign, but the evidence is not available to the laboratory to perform an independent evaluation"}),
        ("BP7", EvidenceCode{category: Benign, strength: Supporting, code: 7, description: "A synonymous (silent) variant for which splicing prediction algorithms predict no impact to the splice consensus sequence nor the creation of a new splice site AND the nucleotide is not highly conserved"}),
        ]);
}



