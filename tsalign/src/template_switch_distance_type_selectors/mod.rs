use std::fmt::Debug;

use clap::ValueEnum;
use compact_genome::interface::{alphabet::Alphabet, sequence::GenomeSequence};
use lib_tsalign::{
    a_star_aligner::{
        template_switch_distance::{
            strategies::{
                node_ord::{AntiDiagonalNodeOrdStrategy, CostOnlyNodeOrdStrategy, NodeOrdStrategy},
                AlignmentStrategySelection,
            },
            Context,
        },
        template_switch_distance_a_star_align,
    },
    cost_table::TemplateSwitchCostTable,
};

use crate::Cli;

#[derive(Clone, ValueEnum)]
pub enum TemplateSwitchNodeOrdStrategy {
    CostOnly,
    AntiDiagonal,
}

pub fn align_a_star_template_switch_distance<
    AlphabetType: Alphabet + Debug + Clone + Eq,
    SubsequenceType: GenomeSequence<AlphabetType, SubsequenceType> + ?Sized,
>(
    cli: Cli,
    reference: &SubsequenceType,
    query: &SubsequenceType,
) {
    align_a_star_template_switch_distance_select_node_ord_strategy(cli, reference, query);
}

fn align_a_star_template_switch_distance_select_node_ord_strategy<
    AlphabetType: Alphabet + Debug + Clone + Eq,
    SubsequenceType: GenomeSequence<AlphabetType, SubsequenceType> + ?Sized,
>(
    cli: Cli,
    reference: &SubsequenceType,
    query: &SubsequenceType,
) {
    match cli.ts_node_ord_strategy {
        TemplateSwitchNodeOrdStrategy::CostOnly => {
            align_a_star_template_switch_distance_call::<_, _, CostOnlyNodeOrdStrategy>(
                cli, reference, query,
            )
        }
        TemplateSwitchNodeOrdStrategy::AntiDiagonal => {
            align_a_star_template_switch_distance_call::<_, _, AntiDiagonalNodeOrdStrategy>(
                cli, reference, query,
            )
        }
    }
}

fn align_a_star_template_switch_distance_call<
    AlphabetType: Alphabet + Debug + Clone + Eq,
    SubsequenceType: GenomeSequence<AlphabetType, SubsequenceType> + ?Sized,
    NodeOrd: NodeOrdStrategy,
>(
    cli: Cli,
    reference: &SubsequenceType,
    query: &SubsequenceType,
) {
    use std::io::Read;

    let mut config_path = cli.configuration_directory.clone();
    config_path.push("tsa_costs.txt");
    let config_file = std::io::BufReader::new(std::fs::File::open(config_path).unwrap());
    let costs = TemplateSwitchCostTable::read_plain(config_file).unwrap();

    #[derive(serde::Deserialize)]
    struct TemplateSwitchConfig {
        left_flank_length: usize,
        right_flank_length: usize,
    }

    let mut config_path = cli.configuration_directory.clone();
    config_path.push("a_star_template_switch.toml");
    let mut config_file = std::io::BufReader::new(std::fs::File::open(config_path).unwrap());
    let mut config = String::new();
    config_file.read_to_string(&mut config).unwrap();
    let config: TemplateSwitchConfig = toml::from_str(&config).unwrap();

    let alignment = template_switch_distance_a_star_align::<
        AlignmentStrategySelection<AlphabetType, NodeOrd>,
        _,
    >(
        reference,
        query,
        Context::<AlphabetType> {
            costs,
            left_flank_length: config.left_flank_length.try_into().unwrap(),
            right_flank_length: config.right_flank_length.try_into().unwrap(),
        },
    );

    println!("{}", alignment);
}
