use std::{fmt::Debug, marker::PhantomData};

use chaining::{ChainingStrategy, NoChainingStrategy};
use compact_genome::interface::{alphabet::Alphabet, sequence::GenomeSequence};
use node_ord::{CostOnlyNodeOrdStrategy, NodeOrdStrategy};
use template_switch_min_length::{
    NoTemplateSwitchMinLengthStrategy, TemplateSwitchMinLengthStrategy,
};

use super::Context;

pub mod chaining;
pub mod node_ord;
pub mod template_switch_min_length;

pub trait AlignmentStrategySelector: Eq + Clone + std::fmt::Debug {
    type Alphabet: Alphabet;
    type NodeOrd: NodeOrdStrategy;
    type TemplateSwitchMinLength: TemplateSwitchMinLengthStrategy;
    type Chaining: ChainingStrategy;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlignmentStrategies<Selector: AlignmentStrategySelector> {
    pub node_ord_strategy: Selector::NodeOrd,
    pub template_switch_min_length_strategy: Selector::TemplateSwitchMinLength,
}

pub trait AlignmentStrategy: Eq + Clone + std::fmt::Debug {
    fn create_root<
        SubsequenceType: GenomeSequence<Strategies::Alphabet, SubsequenceType> + ?Sized,
        Strategies: AlignmentStrategySelector,
    >(
        context: &Context<'_, '_, SubsequenceType, Strategies>,
    ) -> Self;

    fn generate_successor<
        SubsequenceType: GenomeSequence<Strategies::Alphabet, SubsequenceType> + ?Sized,
        Strategies: AlignmentStrategySelector,
    >(
        &self,
        context: &Context<'_, '_, SubsequenceType, Strategies>,
    ) -> Self;
}

impl<Selector: AlignmentStrategySelector> AlignmentStrategy for AlignmentStrategies<Selector> {
    fn create_root<
        SubsequenceType: GenomeSequence<Strategies::Alphabet, SubsequenceType> + ?Sized,
        Strategies: AlignmentStrategySelector,
    >(
        context: &Context<'_, '_, SubsequenceType, Strategies>,
    ) -> Self {
        Self {
            node_ord_strategy: Selector::NodeOrd::create_root(context),
            template_switch_min_length_strategy: Selector::TemplateSwitchMinLength::create_root(
                context,
            ),
        }
    }

    fn generate_successor<
        SubsequenceType: GenomeSequence<Strategies::Alphabet, SubsequenceType> + ?Sized,
        Strategies: AlignmentStrategySelector,
    >(
        &self,
        context: &Context<'_, '_, SubsequenceType, Strategies>,
    ) -> Self {
        Self {
            node_ord_strategy: self.node_ord_strategy.generate_successor(context),
            template_switch_min_length_strategy: self
                .template_switch_min_length_strategy
                .generate_successor(context),
        }
    }
}

pub struct AlignmentStrategySelection<
    AlphabetType: Alphabet,
    NodeOrd: NodeOrdStrategy,
    TemplateSwitchMinLength: TemplateSwitchMinLengthStrategy,
    Chaining: ChainingStrategy,
> {
    phantom_data: PhantomData<(AlphabetType, NodeOrd, TemplateSwitchMinLength, Chaining)>,
}

impl<
        AlphabetType: Alphabet,
        NodeOrd: NodeOrdStrategy,
        TemplateSwitchMinLength: TemplateSwitchMinLengthStrategy,
        Chaining: ChainingStrategy,
    > AlignmentStrategySelector
    for AlignmentStrategySelection<AlphabetType, NodeOrd, TemplateSwitchMinLength, Chaining>
{
    type Alphabet = AlphabetType;
    type NodeOrd = NodeOrd;
    type TemplateSwitchMinLength = TemplateSwitchMinLength;
    type Chaining = Chaining;
}

pub type SimpleAlignmentStrategies<AlphabetType> = AlignmentStrategySelection<
    AlphabetType,
    CostOnlyNodeOrdStrategy,
    NoTemplateSwitchMinLengthStrategy,
    NoChainingStrategy,
>;

impl<
        AlphabetType: Alphabet,
        NodeOrd: NodeOrdStrategy,
        TemplateSwitchMinLength: TemplateSwitchMinLengthStrategy,
        Chaining: ChainingStrategy,
    > Debug
    for AlignmentStrategySelection<AlphabetType, NodeOrd, TemplateSwitchMinLength, Chaining>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AlignmentStrategySelection").finish()
    }
}

impl<
        AlphabetType: Alphabet,
        NodeOrd: NodeOrdStrategy,
        TemplateSwitchMinLength: TemplateSwitchMinLengthStrategy,
        Chaining: ChainingStrategy,
    > Clone
    for AlignmentStrategySelection<AlphabetType, NodeOrd, TemplateSwitchMinLength, Chaining>
{
    fn clone(&self) -> Self {
        Self {
            phantom_data: self.phantom_data,
        }
    }
}

impl<
        AlphabetType: Alphabet,
        NodeOrd: NodeOrdStrategy,
        TemplateSwitchMinLength: TemplateSwitchMinLengthStrategy,
        Chaining: ChainingStrategy,
    > PartialEq
    for AlignmentStrategySelection<AlphabetType, NodeOrd, TemplateSwitchMinLength, Chaining>
{
    fn eq(&self, other: &Self) -> bool {
        self.phantom_data == other.phantom_data
    }
}

impl<
        AlphabetType: Alphabet,
        NodeOrd: NodeOrdStrategy,
        TemplateSwitchMinLength: TemplateSwitchMinLengthStrategy,
        Chaining: ChainingStrategy,
    > Eq for AlignmentStrategySelection<AlphabetType, NodeOrd, TemplateSwitchMinLength, Chaining>
{
}
