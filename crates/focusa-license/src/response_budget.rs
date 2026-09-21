//! Shared response-byte budget for secret-bearing authority transports.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResponseBudget {
    limit: usize,
    consumed: usize,
    declared: Option<u64>,
}

impl ResponseBudget {
    pub(crate) fn new(limit: usize, declared: Option<u64>) -> Result<Self, ResponseBudgetExceeded> {
        if declared.is_some_and(|length| length > limit as u64) {
            return Err(ResponseBudgetExceeded);
        }
        Ok(Self {
            limit,
            consumed: 0,
            declared,
        })
    }

    pub(crate) fn consume(&mut self, bytes: usize) -> Result<(), ResponseBudgetExceeded> {
        let next = self
            .consumed
            .checked_add(bytes)
            .ok_or(ResponseBudgetExceeded)?;
        if next > self.limit {
            return Err(ResponseBudgetExceeded);
        }
        self.consumed = next;
        Ok(())
    }

    pub(crate) fn remaining(&self) -> usize {
        self.limit - self.consumed
    }

    pub(crate) fn initial_capacity(&self) -> usize {
        self.declared.unwrap_or_default().min(self.limit as u64) as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResponseBudgetExceeded;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_oversized_declared_length_before_consumption() {
        assert_eq!(
            ResponseBudget::new(16, Some(17)),
            Err(ResponseBudgetExceeded)
        );
    }

    #[test]
    fn cumulative_chunks_cannot_cross_the_hard_limit() {
        let mut budget = ResponseBudget::new(16, None).expect("budget");
        budget.consume(7).expect("first chunk");
        budget.consume(9).expect("exact limit");
        assert_eq!(budget.remaining(), 0);
        assert_eq!(budget.consume(1), Err(ResponseBudgetExceeded));
    }
}
