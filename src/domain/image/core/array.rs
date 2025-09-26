#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedArray<T>
where
    T: Clone + Default,
{
    rows: usize,
    columns: usize,
    log2_block_size: usize,
    blocks: Vec<T>,
}

impl<T> BlockedArray<T>
where
    T: Clone + Default,
{
    pub fn new(rows: usize, columns: usize, log2_block_size: usize) -> Self {
        let block_size = 1 << log2_block_size;
        let blocks_column = (columns + block_size - 1) >> log2_block_size;
        let blocks_row = (rows + block_size - 1) >> log2_block_size;
        let num_blocks = blocks_column * blocks_row;
        let block_len = block_size * block_size;
        let total_len = num_blocks * block_len;
        let blocks = vec![T::default(); total_len];
        Self {
            rows,
            columns,
            log2_block_size,
            blocks,
        }
    }

    #[inline]
    pub fn get(&self, row: usize, column: usize) -> Option<&T> {
        self.locate_index(row, column)
            .and_then(|flat_idx| self.blocks.get(flat_idx))
    }

    #[inline]
    pub fn get_mut(&mut self, row: usize, column: usize) -> Option<&mut T> {
        self.locate_index(row, column)
            .and_then(move |flat_idx| self.blocks.get_mut(flat_idx))
    }

    pub fn set(&mut self, row: usize, column: usize, value: T) -> bool {
        if let Some(flat_idx) = self.locate_index(row, column) {
            if let Some(cell) = self.blocks.get_mut(flat_idx) {
                *cell = value;
                return true;
            }
        }
        false
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    pub fn columns(&self) -> usize {
        self.columns
    }

    fn locate_index(&self, row: usize, column: usize) -> Option<usize> {
        if row >= self.rows || column >= self.columns {
            return None;
        }

        let block_size = 1 << self.log2_block_size;
        let block_mask = block_size - 1;
        let blocks_per_row = (self.columns + block_size - 1) >> self.log2_block_size;

        let block_row = row >> self.log2_block_size;
        let block_column = column >> self.log2_block_size;
        let block_idx = block_row * blocks_per_row + block_column;

        let inner_row = row & block_mask;
        let inner_column = column & block_mask;
        let offset = (inner_row << self.log2_block_size) + inner_column;

        let block_len = block_size * block_size;
        Some(block_idx * block_len + offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocked_array_get_set_succeeds() {
        let mut arr = BlockedArray::<i32>::new(10, 10, 2);
        assert_eq!(arr.rows(), 10);
        assert_eq!(arr.columns(), 10);

        assert_eq!(arr.get(3, 3), Some(&0));
        assert!(arr.set(3, 3, 42));
        assert_eq!(arr.get(3, 3), Some(&42));

        assert!(arr.set(9, 9, 99));
        assert_eq!(arr.get(9, 9), Some(&99));
    }

    #[test]
    fn blocked_array_get_set_fails_when_out_of_bounds() {
        let mut arr = BlockedArray::<i32>::new(5, 5, 1);
        assert_eq!(arr.get(5, 0), None);
        assert_eq!(arr.get(0, 5), None);
        assert!(!arr.set(5, 0, 1));
        assert!(!arr.set(0, 5, 1));
    }

    #[test]
    fn blocked_array_get_mut_succeeds() {
        let mut arr = BlockedArray::<i32>::new(4, 4, 1);
        if let Some(cell) = arr.get_mut(2, 2) {
            *cell = 7;
        }
        assert_eq!(arr.get(2, 2), Some(&7));
    }
}
