use crate::{
    relocatable,
    vm::errors::{memory_errors::MemoryError, vm_errors::VirtualMachineError},
};
use felt::{Felt, NewFelt};
use num_traits::{FromPrimitive, ToPrimitive};
use std::ops::Add;

#[derive(Eq, Hash, PartialEq, PartialOrd, Clone, Debug)]
pub struct Relocatable {
    pub segment_index: isize,
    pub offset: usize,
}

#[derive(Eq, Hash, PartialEq, PartialOrd, Clone, Debug)]
pub enum MaybeRelocatable {
    RelocatableValue(Relocatable),
    Int(Felt),
}

impl From<(isize, usize)> for Relocatable {
    fn from(index_offset: (isize, usize)) -> Self {
        Relocatable {
            segment_index: index_offset.0,
            offset: index_offset.1,
        }
    }
}

impl From<(isize, usize)> for MaybeRelocatable {
    fn from(index_offset: (isize, usize)) -> Self {
        MaybeRelocatable::RelocatableValue(Relocatable::from(index_offset))
    }
}

impl From<Felt> for MaybeRelocatable {
    fn from(num: Felt) -> Self {
        MaybeRelocatable::Int(num)
    }
}

impl From<&Relocatable> for MaybeRelocatable {
    fn from(rel: &Relocatable) -> Self {
        MaybeRelocatable::RelocatableValue(rel.clone())
    }
}

impl From<&Relocatable> for Relocatable {
    fn from(other: &Relocatable) -> Self {
        other.clone()
    }
}

impl From<&Felt> for MaybeRelocatable {
    fn from(val: &Felt) -> Self {
        MaybeRelocatable::Int(val.clone())
    }
}

impl From<Relocatable> for MaybeRelocatable {
    fn from(rel: Relocatable) -> Self {
        MaybeRelocatable::RelocatableValue(rel)
    }
}

impl Add<usize> for Relocatable {
    type Output = Relocatable;
    fn add(self, other: usize) -> Self {
        relocatable!(self.segment_index, self.offset + other)
    }
}

impl Add<i32> for Relocatable {
    type Output = Relocatable;
    fn add(self, other: i32) -> Self {
        if other >= 0 {
            relocatable!(self.segment_index, self.offset + other as usize)
        } else {
            relocatable!(self.segment_index, self.offset - other.abs() as usize)
        }
    }
}

impl Add<i32> for &Relocatable {
    type Output = Relocatable;
    fn add(self, other: i32) -> Relocatable {
        if other >= 0 {
            relocatable!(self.segment_index, self.offset + other as usize)
        } else {
            relocatable!(self.segment_index, self.offset - other.abs() as usize)
        }
    }
}

impl TryInto<Relocatable> for MaybeRelocatable {
    type Error = MemoryError;
    fn try_into(self) -> Result<Relocatable, MemoryError> {
        match self {
            MaybeRelocatable::RelocatableValue(rel) => Ok(rel),
            _ => Err(MemoryError::AddressNotRelocatable),
        }
    }
}

impl From<&MaybeRelocatable> for MaybeRelocatable {
    fn from(other: &MaybeRelocatable) -> Self {
        other.clone()
    }
}

impl TryFrom<&MaybeRelocatable> for Relocatable {
    type Error = MemoryError;
    fn try_from(other: &MaybeRelocatable) -> Result<Self, MemoryError> {
        match other {
            MaybeRelocatable::RelocatableValue(rel) => Ok(rel.clone()),
            _ => Err(MemoryError::AddressNotRelocatable),
        }
    }
}

impl Relocatable {
    pub fn sub_usize(&self, other: usize) -> Result<Self, VirtualMachineError> {
        if self.offset < other {
            return Err(VirtualMachineError::CantSubOffset(self.offset, other));
        }
        let new_offset = self.offset - other;
        Ok(relocatable!(self.segment_index, new_offset))
    }

    ///Adds a Felt to self
    pub fn add_int(&self, other: &Felt) -> Result<Relocatable, VirtualMachineError> {
        let big_offset = other + self.offset;
        let new_offset = big_offset
            .to_usize()
            .ok_or(VirtualMachineError::OffsetExceeded(big_offset))?;
        Ok(Relocatable {
            segment_index: self.segment_index,
            offset: new_offset,
        })
    }

    /// Adds a MaybeRelocatable to self
    /// Cant add two relocatable values
    pub fn add_maybe(&self, other: &MaybeRelocatable) -> Result<Relocatable, VirtualMachineError> {
        let num_ref = other
            .get_int_ref()
            .map_err(|_| VirtualMachineError::RelocatableAdd)?;

        let big_offset: Felt = num_ref + self.offset;
        let new_offset = big_offset
            .to_usize()
            .ok_or(VirtualMachineError::OffsetExceeded(big_offset))?;
        Ok(Relocatable {
            segment_index: self.segment_index,
            offset: new_offset,
        })
    }

    pub fn sub(&self, other: &Self) -> Result<usize, VirtualMachineError> {
        if self.segment_index != other.segment_index {
            return Err(VirtualMachineError::DiffIndexSub);
        }
        if self.offset < other.offset {
            return Err(VirtualMachineError::CantSubOffset(
                self.offset,
                other.offset,
            ));
        }
        let result = self.offset - other.offset;
        Ok(result)
    }
}

impl MaybeRelocatable {
    /// Adds a Felt to self
    pub fn add_int(&self, other: &Felt) -> Result<MaybeRelocatable, VirtualMachineError> {
        match *self {
            MaybeRelocatable::Int(ref value) => Ok(MaybeRelocatable::Int(value + other)),
            MaybeRelocatable::RelocatableValue(ref rel) => {
                let big_offset = other + rel.offset;
                let new_offset = big_offset
                    .to_usize()
                    .ok_or(VirtualMachineError::OffsetExceeded(big_offset))?;
                Ok(MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: rel.segment_index,
                    offset: new_offset,
                }))
            }
        }
    }

    /// Adds a usize to self
    pub fn add_usize(&self, other: usize) -> MaybeRelocatable {
        match *self {
            MaybeRelocatable::Int(ref value) => MaybeRelocatable::Int(value + other),
            MaybeRelocatable::RelocatableValue(ref rel) => {
                MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: rel.segment_index,
                    offset: rel.offset + other,
                })
            }
        }
    }

    /// Adds a MaybeRelocatable to self
    /// Cant add two relocatable values
    pub fn add(&self, other: &MaybeRelocatable) -> Result<MaybeRelocatable, VirtualMachineError> {
        match (self, other) {
            (&MaybeRelocatable::Int(ref num_a_ref), MaybeRelocatable::Int(num_b)) => {
                Ok(MaybeRelocatable::Int(num_a_ref + num_b))
            }
            (&MaybeRelocatable::RelocatableValue(_), &MaybeRelocatable::RelocatableValue(_)) => {
                Err(VirtualMachineError::RelocatableAdd)
            }
            (&MaybeRelocatable::RelocatableValue(ref rel), &MaybeRelocatable::Int(ref num_ref))
            | (&MaybeRelocatable::Int(ref num_ref), &MaybeRelocatable::RelocatableValue(ref rel)) =>
            {
                let big_offset: Felt = num_ref + rel.offset;
                let new_offset = big_offset
                    .to_usize()
                    .ok_or(VirtualMachineError::OffsetExceeded(big_offset))?;
                Ok(MaybeRelocatable::RelocatableValue(Relocatable {
                    segment_index: rel.segment_index,
                    offset: new_offset,
                }))
            }
        }
    }
    /// Substracts two MaybeRelocatable values and returns the result as a MaybeRelocatable value.
    /// Only values of the same type may be substracted.
    /// Relocatable values can only be substracted if they belong to the same segment.
    pub fn sub(&self, other: &MaybeRelocatable) -> Result<MaybeRelocatable, VirtualMachineError> {
        match (self, other) {
            (&MaybeRelocatable::Int(ref num_a), &MaybeRelocatable::Int(ref num_b)) => {
                Ok(MaybeRelocatable::Int(num_a - num_b))
            }
            (
                MaybeRelocatable::RelocatableValue(rel_a),
                MaybeRelocatable::RelocatableValue(rel_b),
            ) => {
                if rel_a.segment_index == rel_b.segment_index {
                    return Ok(MaybeRelocatable::from(Felt::new(
                        rel_a.offset - rel_b.offset,
                    )));
                }
                Err(VirtualMachineError::DiffIndexSub)
            }
            (MaybeRelocatable::RelocatableValue(rel_a), MaybeRelocatable::Int(ref num_b)) => {
                Ok(MaybeRelocatable::from((
                    rel_a.segment_index,
                    (rel_a.offset - num_b)
                        .to_usize()
                        .ok_or_else(|| VirtualMachineError::OffsetExceeded(rel_a.offset - num_b))?,
                )))
            }
            _ => Err(VirtualMachineError::NotImplemented),
        }
    }

    /// Performs mod floor for a MaybeRelocatable::Int with Felt.
    /// When self is a Relocatable it just returns a clone of itself.
    /*pub fn mod_floor(&self, other: &Felt) -> Result<MaybeRelocatable, VirtualMachineError> {
        match self {
            MaybeRelocatable::Int(value) => Ok(MaybeRelocatable::Int(value.mod_floor(other))),
            MaybeRelocatable::RelocatableValue(_) => Ok(self.clone()),
        }
    }*/

    /// Performs integer division and module on a MaybeRelocatable::Int by another
    /// MaybeRelocatable::Int and returns the quotient and reminder.
    pub fn divmod(
        &self,
        other: &MaybeRelocatable,
    ) -> Result<(MaybeRelocatable, MaybeRelocatable), VirtualMachineError> {
        match (self, other) {
            (&MaybeRelocatable::Int(ref val), &MaybeRelocatable::Int(ref div)) => Ok((
                MaybeRelocatable::from(val / div.clone()),
                MaybeRelocatable::from(val.mod_floor(div)),
            )),
            _ => Err(VirtualMachineError::NotImplemented),
        }
    }

    //Returns reference to Felt inside self if Int variant or Error if RelocatableValue variant
    pub fn get_int_ref(&self) -> Result<&Felt, VirtualMachineError> {
        match self {
            MaybeRelocatable::Int(num) => Ok(num),
            MaybeRelocatable::RelocatableValue(_) => {
                Err(VirtualMachineError::ExpectedInteger(self.clone()))
            }
        }
    }

    //Returns reference to Relocatable inside self if Relocatable variant or Error if Int variant
    pub fn get_relocatable(&self) -> Result<&Relocatable, VirtualMachineError> {
        match self {
            MaybeRelocatable::RelocatableValue(rel) => Ok(rel),
            MaybeRelocatable::Int(_) => Err(VirtualMachineError::ExpectedRelocatable(self.clone())),
        }
    }
}

impl<'a> Add<usize> for &'a Relocatable {
    type Output = Relocatable;

    fn add(self, other: usize) -> Self::Output {
        Relocatable {
            segment_index: self.segment_index,
            offset: self.offset + other,
        }
    }
}

/// Turns a MaybeRelocatable into a Felt value.
/// If the value is an Int, it will extract the Felt value from it.
/// If the value is Relocatable, it will return an error since it should've already been relocated.
pub fn relocate_value(
    value: MaybeRelocatable,
    relocation_table: &Vec<usize>,
) -> Result<Felt, MemoryError> {
    match value {
        MaybeRelocatable::Int(num) => Ok(num),
        MaybeRelocatable::RelocatableValue(relocatable) => {
            let (segment_index, offset) = if relocatable.segment_index >= 0 {
                (
                    relocatable.segment_index as usize,
                    relocatable.offset as usize,
                )
            } else {
                return Err(MemoryError::TemporarySegmentInRelocation(
                    relocatable.segment_index,
                ));
            };

            if relocation_table.len() <= segment_index {
                return Err(MemoryError::Relocation);
            }
            Felt::from_usize(relocation_table[segment_index] + offset)
                .ok_or(MemoryError::Relocation)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{felt_str, relocatable, utils::test_utils::mayberelocatable};
    use num_traits::pow;

    #[test]
    fn add_bigint_to_int() {
        let addr = MaybeRelocatable::from(Felt::new(7i32));
        let added_addr = addr.add_int(&Felt::new(2i32));
        assert_eq!(Ok(MaybeRelocatable::Int(Felt::new(9i32))), added_addr);
    }

    #[test]
    fn add_usize_to_int() {
        let addr = MaybeRelocatable::from(Felt::new(7_i32));
        let added_addr = addr.add_usize(2);
        assert_eq!(MaybeRelocatable::Int(Felt::new(9_i32)), added_addr);
    }

    #[test]
    fn add_bigint_to_relocatable() {
        let addr = MaybeRelocatable::RelocatableValue(relocatable!(7, 65));
        let added_addr = addr.add_int(&Felt::new(2));
        assert_eq!(Ok(MaybeRelocatable::from((7, 67))), added_addr);
    }

    #[test]
    fn add_int_mod_offset_exceeded() {
        let addr = MaybeRelocatable::from((0, 0));
        let error = addr.add_int(&felt_str!("18446744073709551616"));
        assert_eq!(
            error,
            Err(VirtualMachineError::OffsetExceeded(felt_str!(
                "18446744073709551616"
            )))
        );
        assert_eq!(
            error.unwrap_err().to_string(),
            "Offset 18446744073709551616 exeeds maximum offset value"
        );
    }

    #[test]
    fn add_usize_to_relocatable() {
        let addr = MaybeRelocatable::RelocatableValue(relocatable!(7, 65));
        let added_addr = addr.add_int(&Felt::new(2));
        assert_eq!(Ok(MaybeRelocatable::from((7, 67))), added_addr);
    }

    #[test]
    fn add_bigint_to_int_prime_mod() {
        let addr = MaybeRelocatable::Int(Felt::new(vec![
            43680, 0, 0, 0, 0, 0, 0, 2013265920, 4294967289, 4294967295, 4294967295, 4294967295,
            4294967295, 4294967295, 4294967295, 1048575,
        ]));
        let added_addr = addr.add_int_mod(
            &Felt::one(),
            &Felt::new(
                Sign::Plus,
                vec![
                    4294967089, 4294967295, 4294967295, 4294967295, 4294967295, 4294967295,
                    4294967295, 67108863,
                ],
            ),
        );
        assert_eq!(Ok(MaybeRelocatable::Int(Felt::new(4))), added_addr);
    }

    #[test]
    fn add_bigint_to_relocatable_prime() {
        let addr = MaybeRelocatable::RelocatableValue(relocatable!(1, 9));
        let added_addr = addr.add_int_mod(
            &Felt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
            &Felt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(1, 9))),
            added_addr
        );
    }

    #[test]
    fn add_int_to_int() {
        let addr_a = &MaybeRelocatable::from(Felt::new(7));
        let addr_b = &MaybeRelocatable::from(Felt::new(17));
        let added_addr = addr_a.add_mod(addr_b, &Felt::new(71));
        assert_eq!(Ok(MaybeRelocatable::from(Felt::new(24))), added_addr);
    }

    #[test]
    fn add_int_to_int_prime() {
        let addr_a =
            &MaybeRelocatable::Int(Felt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]));
        let addr_b = &MaybeRelocatable::from(Felt::new(17));
        let added_addr = addr_a.add_mod(
            addr_b,
            &Felt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );
        assert_eq!(Ok(MaybeRelocatable::from(Felt::new(17))), added_addr);
    }

    #[test]
    fn add_relocatable_to_relocatable_should_fail() {
        let addr_a = &MaybeRelocatable::from((7, 5));
        let addr_b = &MaybeRelocatable::RelocatableValue(relocatable!(7, 10));
        let error = addr_a.add_mod(addr_b, &Felt::new(17));
        assert_eq!(error, Err(VirtualMachineError::RelocatableAdd));
        assert_eq!(
            error.unwrap_err().to_string(),
            "Cannot add two relocatable values"
        );
    }

    #[test]
    fn add_int_to_relocatable() {
        let addr_a = &MaybeRelocatable::from((7, 7));
        let addr_b = &MaybeRelocatable::from(Felt::new(10));
        let added_addr = addr_a.add_mod(addr_b, &Felt::new(21));
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(7, 17))),
            added_addr
        );
    }

    #[test]
    fn add_relocatable_to_int() {
        let addr_a = &MaybeRelocatable::from(Felt::new(10));
        let addr_b = &MaybeRelocatable::RelocatableValue(relocatable!(7, 7));
        let added_addr = addr_a.add_mod(addr_b, &Felt::new(21));
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(7, 17))),
            added_addr
        );
    }

    #[test]
    fn add_int_to_relocatable_prime() {
        let addr_a = &MaybeRelocatable::from((7, 14));
        let addr_b =
            &MaybeRelocatable::Int(Felt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]));
        let added_addr = addr_a.add_mod(
            addr_b,
            &Felt::new(Sign::Plus, vec![1, 0, 0, 0, 0, 0, 17, 134217728]),
        );
        assert_eq!(
            Ok(MaybeRelocatable::RelocatableValue(relocatable!(7, 14))),
            added_addr
        );
    }

    #[test]
    fn add_int_rel_int_offset_exceeded() {
        let addr = MaybeRelocatable::from((0, 0));
        let error = addr.add_mod(
            &MaybeRelocatable::from(felt_str!("18446744073709551616")),
            &felt_str!("18446744073709551617"),
        );
        assert_eq!(
            error,
            Err(VirtualMachineError::OffsetExceeded(felt_str!(
                "18446744073709551616"
            )))
        );
    }

    #[test]
    fn add_int_int_rel_offset_exceeded() {
        let addr = MaybeRelocatable::Int(felt_str!("18446744073709551616"));
        let relocatable = Relocatable {
            offset: 0,
            segment_index: 0,
        };
        let error = addr.add_mod(
            &MaybeRelocatable::RelocatableValue(relocatable),
            &felt_str!("18446744073709551617"),
        );
        assert_eq!(
            error,
            Err(VirtualMachineError::OffsetExceeded(felt_str!(
                "18446744073709551616"
            )))
        );
    }

    #[test]
    fn sub_int_from_int() {
        let addr_a = &MaybeRelocatable::from(Felt::new(7));
        let addr_b = &MaybeRelocatable::from(Felt::new(5));
        let sub_addr = addr_a.sub(addr_b, &Felt::new(23));
        assert_eq!(Ok(MaybeRelocatable::from(Felt::new(2))), sub_addr);
    }

    #[test]
    fn sub_relocatable_from_relocatable_same_offset() {
        let addr_a = &MaybeRelocatable::from((7, 17));
        let addr_b = &MaybeRelocatable::from((7, 7));
        let sub_addr = addr_a.sub(addr_b, &Felt::new(23));
        assert_eq!(Ok(MaybeRelocatable::from(Felt::new(10))), sub_addr);
    }

    #[test]
    fn sub_relocatable_from_relocatable_diff_offset() {
        let addr_a = &MaybeRelocatable::from((7, 17));
        let addr_b = &MaybeRelocatable::from((8, 7));
        let error = addr_a.sub(addr_b, &Felt::new(23));
        assert_eq!(error, Err(VirtualMachineError::DiffIndexSub));
        assert_eq!(
            error.unwrap_err().to_string(),
            "Can only subtract two relocatable values of the same segment"
        );
    }

    #[test]
    fn sub_int_addr_ref_from_relocatable_addr_ref() {
        let addr_a = &MaybeRelocatable::from((7, 17));
        let addr_b = &MaybeRelocatable::from(Felt::new(5));
        let addr_c = addr_a.sub(addr_b, &Felt::new(23));
        assert_eq!(addr_c, Ok(MaybeRelocatable::from((7, 12))));
    }

    #[test]
    fn sub_rel_to_int_error() {
        let a = &MaybeRelocatable::from(Felt::new(7));
        let b = &MaybeRelocatable::from((7, 10));
        assert_eq!(
            Err(VirtualMachineError::NotImplemented),
            a.sub(b, &Felt::new(23))
        );
    }

    #[test]
    fn divmod_working() {
        let value = &MaybeRelocatable::from(Felt::new(10));
        let div = &MaybeRelocatable::from(Felt::new(3));
        let (q, r) = value.divmod(div).expect("Unexpected error in divmod");
        assert_eq!(q, MaybeRelocatable::from(Felt::new(3)));
        assert_eq!(r, MaybeRelocatable::from(Felt::one()));
    }

    #[test]
    fn divmod_bad_type() {
        let value = &MaybeRelocatable::from(Felt::new(10));
        let div = &MaybeRelocatable::from((2, 7));
        assert_eq!(value.divmod(div), Err(VirtualMachineError::NotImplemented));
    }

    #[test]
    fn mod_floor_int() {
        let num = MaybeRelocatable::Int(Felt::new(7));
        let div = Felt::new(5);
        let expected_rem = MaybeRelocatable::Int(Felt::new(2));
        assert_eq!(num.mod_floor(&div), Ok(expected_rem));
    }

    #[test]
    fn mod_floor_relocatable() {
        let value = &MaybeRelocatable::from((2, 7));
        let div = Felt::new(5);
        assert_eq!(value.mod_floor(&div), Ok(value.clone()));
    }

    #[test]
    fn relocate_relocatable_value() {
        let value = MaybeRelocatable::from((2, 7));
        let relocation_table = vec![1, 2, 5];
        assert_eq!(relocate_value(value, &relocation_table), Ok(Felt::new(12)));
    }

    #[test]
    fn relocate_relocatable_in_temp_segment_value() {
        let value = MaybeRelocatable::from((-1, 7));
        let relocation_table = vec![1, 2, 5];
        assert_eq!(
            relocate_value(value, &relocation_table),
            Err(MemoryError::TemporarySegmentInRelocation(-1)),
        );
    }

    #[test]
    fn relocate_relocatable_in_temp_segment_value_with_offset() {
        let value = MaybeRelocatable::from((-1, 7));
        let relocation_table = vec![1, 2, 5];
        assert_eq!(
            relocate_value(value, &relocation_table),
            Err(MemoryError::TemporarySegmentInRelocation(-1)),
        );
    }

    #[test]
    fn relocate_relocatable_in_temp_segment_value_error() {
        let value = MaybeRelocatable::from((-1, 7));
        let relocation_table = vec![1, 2, 5];
        assert_eq!(
            relocate_value(value, &relocation_table),
            Err(MemoryError::TemporarySegmentInRelocation(-1))
        );
    }

    #[test]
    fn relocate_int_value() {
        let value = MaybeRelocatable::from(Felt::new(7));
        let relocation_table = vec![1, 2, 5];
        assert_eq!(relocate_value(value, &relocation_table), Ok(Felt::new(7)));
    }

    #[test]
    fn relocate_relocatable_value_no_relocation() {
        let value = MaybeRelocatable::from((2, 7));
        let relocation_table = vec![1, 2];
        assert_eq!(
            relocate_value(value, &relocation_table),
            Err(MemoryError::Relocation)
        );
    }

    #[test]
    fn relocatable_add_int_mod_ok() {
        assert_eq!(
            Ok(relocatable!(1, 6)),
            relocatable!(1, 2).add_int_mod(&Felt::new(4), &Felt::new(71))
        );
        assert_eq!(
            Ok(relocatable!(3, 2)),
            relocatable!(3, 2).add_int_mod(&Felt::zero(), &Felt::new(71))
        );
        assert_eq!(
            Ok(relocatable!(9, 12)),
            relocatable!(9, 48).add_int_mod(&Felt::new(35), &Felt::new(71))
        );
    }

    #[test]
    fn relocatable_add_int_mod_offset_exceeded_error() {
        assert_eq!(
            Err(VirtualMachineError::OffsetExceeded(
                Felt::new(usize::MAX) + 1usize
            )),
            relocatable!(0, 0)
                .add_int_mod(&(Felt::new(usize::MAX) + 1), &(Felt::new(usize::MAX) + 2))
        );
    }

    #[test]
    fn relocatable_add_i32() {
        let reloc = relocatable!(1, 5);

        assert_eq!(&reloc + 3, relocatable!(1, 8));
        assert_eq!(&reloc + (-3), relocatable!(1, 2));
    }

    #[test]
    #[should_panic]
    fn relocatable_add_i32_with_overflow() {
        let reloc = relocatable!(1, 1);

        let _panic = &reloc + (-3);
    }

    #[test]
    fn mayberelocatable_try_into_reloctable() {
        let address = mayberelocatable!(1, 2);
        assert_eq!(Ok(relocatable!(1, 2)), address.try_into());

        let value = mayberelocatable!(1);
        let err: Result<Relocatable, _> = value.try_into();
        assert_eq!(Err(MemoryError::AddressNotRelocatable), err)
    }

    #[test]
    fn relocatable_sub_rel_test() {
        let reloc = relocatable!(7, 6);

        assert_eq!(Ok(1), reloc.sub_rel(&relocatable!(7, 5)));
        assert_eq!(
            Err(VirtualMachineError::CantSubOffset(6, 9)),
            reloc.sub_rel(&relocatable!(7, 9))
        );
    }

    #[test]
    fn sub_rel_different_indexes() {
        let a = relocatable!(7, 6);
        let b = relocatable!(8, 6);

        assert_eq!(Err(VirtualMachineError::DiffIndexSub), a.sub_rel(&b));
    }

    #[test]
    fn add_maybe_mod_ok() {
        assert_eq!(
            Ok(relocatable!(1, 2)),
            relocatable!(1, 0).add_maybe_mod(&mayberelocatable!(2), &Felt::new(71))
        );
        assert_eq!(
            Ok(relocatable!(0, 58)),
            relocatable!(0, 29).add_maybe_mod(&mayberelocatable!(100), &Felt::new(71))
        );
        assert_eq!(
            Ok(relocatable!(2, 45)),
            relocatable!(2, 12).add_maybe_mod(&mayberelocatable!(104), &Felt::new(71))
        );

        assert_eq!(
            Ok(relocatable!(1, 0)),
            relocatable!(1, 0).add_maybe_mod(&mayberelocatable!(0), &Felt::new(71))
        );
        assert_eq!(
            Ok(relocatable!(1, 2)),
            relocatable!(1, 2).add_maybe_mod(&mayberelocatable!(71), &Felt::new(71))
        );

        assert_eq!(
            Ok(relocatable!(14, 0)),
            relocatable!(14, (71 * 12))
                .add_maybe_mod(&mayberelocatable!((pow(71, 3))), &Felt::new(71))
        );
    }

    #[test]
    fn add_maybe_mod_add_two_relocatable_error() {
        assert_eq!(
            Err(VirtualMachineError::RelocatableAdd),
            relocatable!(1, 0).add_maybe_mod(&mayberelocatable!(1, 2), &Felt::new(71))
        );
    }

    #[test]
    fn add_maybe_mod_offset_exceeded_error() {
        assert_eq!(
            Err(VirtualMachineError::OffsetExceeded(
                Felt::new(usize::MAX) + 1
            )),
            relocatable!(1, 0).add_maybe_mod(
                &mayberelocatable!(Felt::new(usize::MAX) + 1),
                &(Felt::new(usize::MAX) + 8)
            )
        );
    }

    #[test]
    fn get_relocatable_test() {
        assert_eq!(
            Ok(&relocatable!(1, 2)),
            mayberelocatable!(1, 2).get_relocatable()
        );
        assert_eq!(
            Err(VirtualMachineError::ExpectedRelocatable(mayberelocatable!(
                3
            ))),
            mayberelocatable!(3).get_relocatable()
        )
    }
}
