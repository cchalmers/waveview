#[derive(Debug, Clone)]
pub struct List<T> {
    items: Vec<T>,
    cummulate_heights: Vec<f32>,
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            items: Vec::new(),
            cummulate_heights: Vec::new(),
        }
    }

    pub fn total_height(&self) -> f32 {
        *self.cummulate_heights.last().unwrap_or(&0.0)
    }

    pub fn push(&mut self, item: T, height: f32) {
        assert!(height >= 0.0);
        self.items.push(item);
        self.cummulate_heights.push(self.total_height() + height);
    }

    pub fn lower_bound(&self, y: f32) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        Some(self.cummulate_heights.partition_point(|h| *h <= y))
    }

    pub fn get_item(&self, y: f32) -> Option<&T> {
        self.lower_bound(y).map(|i| &self.items[i])
    }

    pub fn swap(&mut self, ix_a: usize, ix_b: usize) {
        let a_height = self.cummulate_heights[ix_a];
        let b_height = self.cummulate_heights[ix_b];
        self.items.swap(ix_a, ix_b);
        self.cummulate_heights.swap(ix_a, ix_b);
        if a_height == b_height {
            return;
        }
        if ix_a < ix_b {
            for i in ix_a..ix_b {
                self.cummulate_heights[i] -= b_height - a_height;
            }
        } else {
            for i in ix_b..ix_a {
                self.cummulate_heights[i] += b_height - a_height;
            }
        }
    }

    pub fn change_height(&mut self, ix: usize, height: f32) {
        let old_height = self.cummulate_heights[ix];
        self.cummulate_heights[ix] = height;
        if old_height == height {
            return;
        }
        self.cummulate_heights[ix..]
            .iter_mut()
            .for_each(|h| *h += height - old_height);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_list() {
        let mut list = List::new();
        list.push(1, 1.0);
        list.push(2, 2.0);
        list.push(3, 3.0);
        list.push(4, 4.0);

        assert_eq!(list.lower_bound(0.0), Some(0));
        assert_eq!(list.lower_bound(0.5), Some(0));
        assert_eq!(list.lower_bound(1.0), Some(1));
        assert_eq!(list.lower_bound(1.5), Some(1));
        assert_eq!(list.lower_bound(3.0), Some(2));
        assert_eq!(list.lower_bound(3.5), Some(2));
        assert_eq!(list.lower_bound(6.0), Some(3));
        assert_eq!(list.lower_bound(6.5), Some(3));
        assert_eq!(list.lower_bound(10.0), Some(4));
        assert_eq!(list.lower_bound(300.0), Some(4));
    }
}
