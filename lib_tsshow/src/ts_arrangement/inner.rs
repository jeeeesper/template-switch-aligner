use std::iter;

use lib_tsalign::a_star_aligner::template_switch_distance::{
    AlignmentType, TemplateSwitchSecondary,
};
use log::trace;
use tagged_vec::TaggedVec;

use crate::ts_arrangement::character::Char;

use super::{
    complement::TsComplementArrangement,
    index_types::{ArrangementColumn, SourceColumn, TsInnerIdentifier},
    source::{RemovedHiddenChars, SourceChar, TsSourceArrangement},
    template_switch::TemplateSwitch,
};

pub struct TsInnerArrangement {
    reference_inners: TaggedVec<TsInnerIdentifier, TsInner>,
    query_inners: TaggedVec<TsInnerIdentifier, TsInner>,
}

pub struct TsInner {
    sequence: TaggedVec<ArrangementColumn, InnerChar>,
    template_switch: TemplateSwitch,
}

#[derive(Debug, Clone, Copy)]
pub enum InnerChar {
    Inner {
        column: SourceColumn,
        lower_case: bool,
        copy_depth: Option<usize>,
    },
    Gap {
        copy_depth: Option<usize>,
    },
    Blank,
}

impl TsInnerArrangement {
    pub fn new(
        source_arrangement: &mut TsSourceArrangement,
        complement_arrangement: &mut TsComplementArrangement,
        template_switches: Vec<TemplateSwitch>,
    ) -> Self {
        let mut result = Self {
            reference_inners: Default::default(),
            query_inners: Default::default(),
        };

        for ts in template_switches {
            trace!("source_inner: {:?}", ts.inner);

            let (mut sp2_secondary, sp3_secondary) = match ts.secondary {
                TemplateSwitchSecondary::Reference => (
                    source_arrangement.reference_source_to_arrangement_column(ts.sp2_secondary),
                    source_arrangement.reference_source_to_arrangement_column(ts.sp3_secondary),
                ),
                TemplateSwitchSecondary::Query => (
                    source_arrangement.query_source_to_arrangement_column(ts.sp2_secondary),
                    source_arrangement.query_source_to_arrangement_column(ts.sp3_secondary),
                ),
            };

            let mut source_inner = ts.inner.iter().rev().copied();
            let mut inner = TaggedVec::<ArrangementColumn, _>::default();
            inner.extend(iter::repeat_n(InnerChar::Blank, sp3_secondary.into()));

            let mut current_arrangement_column = sp3_secondary;
            for alignment_type in ts.inner_alignment.iter_flat_cloned().rev() {
                match alignment_type {
                    AlignmentType::SecondaryInsertion => {
                        loop {
                            let c = complement_arrangement.secondary_complement(ts.secondary)
                                [current_arrangement_column];

                            if c.is_gap() || c.is_source_char() {
                                break;
                            }

                            inner.push(InnerChar::Blank);
                            current_arrangement_column += 1;
                        }

                        if !complement_arrangement.secondary_complement(ts.secondary)
                            [current_arrangement_column]
                            .is_gap()
                        {
                            complement_arrangement.insert_secondary_complement_gap(
                                ts.secondary,
                                current_arrangement_column,
                            );

                            source_arrangement.insert_blank(current_arrangement_column);
                            for existing_inner in result
                                .reference_inners
                                .iter_values_mut()
                                .chain(&mut result.query_inners)
                            {
                                existing_inner
                                    .sequence
                                    .insert(current_arrangement_column, InnerChar::Blank);
                            }

                            sp2_secondary += 1;
                        }

                        inner.push(source_inner.next().unwrap().into());
                        current_arrangement_column += 1;
                    }
                    AlignmentType::SecondaryDeletion => {
                        while !complement_arrangement.secondary_complement(ts.secondary)
                            [current_arrangement_column]
                            .is_source_char()
                        {
                            inner.push(InnerChar::Blank);
                            current_arrangement_column += 1;
                        }

                        complement_arrangement
                            .show_secondary_character(ts.secondary, current_arrangement_column);
                        inner.push(InnerChar::Gap {
                            copy_depth: source_arrangement.secondary(ts.secondary)
                                [current_arrangement_column]
                                .copy_depth(),
                        });
                        current_arrangement_column += 1;
                    }
                    AlignmentType::SecondarySubstitution | AlignmentType::SecondaryMatch => {
                        while !source_arrangement.secondary(ts.secondary)
                            [current_arrangement_column]
                            .is_source_char()
                        {
                            inner.push(InnerChar::Blank);
                            current_arrangement_column += 1;
                        }

                        complement_arrangement
                            .show_secondary_character(ts.secondary, current_arrangement_column);

                        let mut inner_char: InnerChar = source_inner.next().unwrap().into();
                        if alignment_type == AlignmentType::SecondarySubstitution {
                            complement_arrangement
                                .secondary_to_lower_case(ts.secondary, current_arrangement_column);
                            inner_char.to_lower_case();
                        }

                        inner.push(inner_char);
                        current_arrangement_column += 1;
                    }
                    _ => unreachable!(),
                }
            }

            // We skip further secondary non-source chars for the assertion below.
            while !source_arrangement.secondary(ts.secondary)[current_arrangement_column]
                .is_source_char()
            {
                current_arrangement_column += 1;
            }
            assert_eq!(current_arrangement_column, sp2_secondary);

            let suffix_blanks =
                iter::repeat_n(InnerChar::Blank, source_arrangement.reference().len())
                    .skip(inner.len());
            inner.extend(suffix_blanks);

            match ts.secondary {
                TemplateSwitchSecondary::Reference => {
                    result.reference_inners.push(TsInner::new(inner, ts))
                }
                TemplateSwitchSecondary::Query => result.query_inners.push(TsInner::new(inner, ts)),
            };
        }

        result
    }

    pub fn remove_columns(
        &mut self,
        columns: impl IntoIterator<Item = ArrangementColumn> + Clone,
        removed_hidden_chars: &RemovedHiddenChars,
    ) {
        for inner in self
            .reference_inners
            .iter_values_mut()
            .chain(&mut self.query_inners)
        {
            inner.sequence.remove_multi(columns.clone());
            inner
                .template_switch
                .remove_hidden_chars(removed_hidden_chars);
        }
    }

    pub fn reference_inners(&self) -> &TaggedVec<TsInnerIdentifier, TsInner> {
        &self.reference_inners
    }

    pub fn query_inners(&self) -> &TaggedVec<TsInnerIdentifier, TsInner> {
        &self.query_inners
    }

    pub fn reference_inner_first_non_blank_column(
        &self,
        inner_identifier: TsInnerIdentifier,
    ) -> ArrangementColumn {
        Self::inner_first_non_blank_column(&self.reference_inners[inner_identifier].sequence)
    }

    pub fn reference_inner_last_non_blank_column(
        &self,
        inner_identifier: TsInnerIdentifier,
    ) -> ArrangementColumn {
        Self::inner_last_non_blank_column(&self.reference_inners[inner_identifier].sequence)
    }

    pub fn query_inner_first_non_blank_column(
        &self,
        inner_identifier: TsInnerIdentifier,
    ) -> ArrangementColumn {
        Self::inner_first_non_blank_column(&self.query_inners[inner_identifier].sequence)
    }

    pub fn query_inner_last_non_blank_column(
        &self,
        inner_identifier: TsInnerIdentifier,
    ) -> ArrangementColumn {
        Self::inner_last_non_blank_column(&self.query_inners[inner_identifier].sequence)
    }

    fn inner_first_non_blank_column(
        sequence: &TaggedVec<ArrangementColumn, InnerChar>,
    ) -> ArrangementColumn {
        sequence
            .iter()
            .find(|(_, c)| !c.is_blank())
            .map(|(i, _)| i)
            .unwrap_or(sequence.len().into())
    }

    fn inner_last_non_blank_column(
        sequence: &TaggedVec<ArrangementColumn, InnerChar>,
    ) -> ArrangementColumn {
        sequence
            .iter()
            .rev()
            .find(|(_, c)| !c.is_blank())
            .map(|(i, _)| i)
            .unwrap() // If None, would need to return -1 here, but return value is unsigned.
    }
}

impl TsInner {
    pub fn new(
        sequence: TaggedVec<ArrangementColumn, InnerChar>,
        template_switch: TemplateSwitch,
    ) -> Self {
        Self {
            sequence,
            template_switch,
        }
    }

    pub fn sequence(&self) -> &TaggedVec<ArrangementColumn, InnerChar> {
        &self.sequence
    }

    pub fn template_switch(&self) -> &TemplateSwitch {
        &self.template_switch
    }
}

impl InnerChar {
    pub fn to_lower_case(&mut self) {
        match self {
            InnerChar::Inner { lower_case, .. } => *lower_case = true,
            InnerChar::Gap { .. } | InnerChar::Blank => panic!("Not lowercasable"),
        }
    }
}

impl From<SourceChar> for InnerChar {
    fn from(value: SourceChar) -> Self {
        match value {
            SourceChar::Source {
                column,
                lower_case,
                copy_depth,
            } => Self::Inner {
                column,
                lower_case,
                copy_depth,
            },
            SourceChar::Hidden { .. } => {
                panic!("Cannot be translated into InnerChar")
            }
            SourceChar::Gap { copy_depth } => Self::Gap { copy_depth },
            SourceChar::Spacer | SourceChar::Blank => Self::Blank,
        }
    }
}

impl Char for InnerChar {
    fn source_column(&self) -> SourceColumn {
        match self {
            InnerChar::Inner { column, .. } => *column,
            InnerChar::Gap { .. } | InnerChar::Blank => panic!("Has no source column"),
        }
    }

    fn is_char(&self) -> bool {
        matches!(self, Self::Inner { .. })
    }

    fn is_gap(&self) -> bool {
        matches!(self, Self::Gap { .. })
    }

    fn is_spacer(&self) -> bool {
        false
    }

    fn is_blank(&self) -> bool {
        matches!(self, Self::Blank)
    }

    fn is_source_char(&self) -> bool {
        self.is_char()
    }

    fn is_hidden(&self) -> bool {
        false
    }
}
