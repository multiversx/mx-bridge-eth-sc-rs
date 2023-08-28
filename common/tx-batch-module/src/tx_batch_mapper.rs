use multiversx_sc::{
    api::{ErrorApiImpl, StorageMapperApi},
    storage::{
        mappers::{StorageClearable, StorageMapper, VecMapper},
        StorageKey,
    },
};
use transaction::Transaction;

static EMPTY_VEC_ERR_MSG: &[u8] = b"Empty vec";

pub struct TxBatchMapper<SA>
where
    SA: StorageMapperApi,
{
    vec_mapper: VecMapper<SA, Transaction<SA>>,
    vec_len: usize,
    first_tx: Option<Transaction<SA>>,
    last_tx: Option<Transaction<SA>>,
}

impl<SA> StorageMapper<SA> for TxBatchMapper<SA>
where
    SA: StorageMapperApi,
{
    fn new(base_key: StorageKey<SA>) -> Self {
        let vec_mapper = VecMapper::new(base_key);
        let vec_len = vec_mapper.len();

        let (first_tx, last_tx) = if vec_len > 0 {
            (Some(vec_mapper.get(1)), Some(vec_mapper.get(vec_len)))
        } else {
            (None, None)
        };

        TxBatchMapper {
            vec_mapper,
            vec_len,
            first_tx,
            last_tx,
        }
    }
}

impl<SA> StorageClearable for TxBatchMapper<SA>
where
    SA: StorageMapperApi,
{
    fn clear(&mut self) {
        self.vec_mapper.clear();
    }
}

impl<SA> TxBatchMapper<SA>
where
    SA: StorageMapperApi,
{
    #[inline]
    pub fn len(&self) -> usize {
        self.vec_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get_first_tx(&self) -> &Transaction<SA> {
        if self.is_empty() {
            SA::error_api_impl().signal_error(EMPTY_VEC_ERR_MSG);
        }

        unsafe { self.first_tx.as_ref().unwrap_unchecked() }
    }

    pub fn get_last_tx(&self) -> &Transaction<SA> {
        if self.is_empty() {
            SA::error_api_impl().signal_error(EMPTY_VEC_ERR_MSG);
        }

        unsafe { self.last_tx.as_ref().unwrap_unchecked() }
    }

    pub fn push(&mut self, tx: Transaction<SA>) {
        if self.is_empty() {
            self.first_tx = Some(tx.clone());
        }

        self.vec_mapper.push(&tx);
        self.vec_len += 1;
        self.last_tx = Some(tx);
    }

    /// Provides a forward iterator.
    pub fn iter(&self) -> Iter<'_, SA> {
        Iter::new(self)
    }
}

pub struct Iter<'a, SA>
where
    SA: StorageMapperApi,
{
    index: usize,
    mapper: &'a TxBatchMapper<SA>,
}

impl<'a, SA> Iter<'a, SA>
where
    SA: StorageMapperApi,
{
    fn new(mapper: &'a TxBatchMapper<SA>) -> Iter<'a, SA> {
        Iter { index: 1, mapper }
    }
}

impl<'a, SA> Iterator for Iter<'a, SA>
where
    SA: StorageMapperApi,
{
    type Item = Transaction<SA>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current_index = self.index;
        if current_index > self.mapper.len() {
            return None;
        }

        self.index += 1;

        if current_index == 1 {
            return Some(self.mapper.get_first_tx().clone());
        }
        if current_index == self.mapper.len() {
            return Some(self.mapper.get_last_tx().clone());
        }

        Some(self.mapper.vec_mapper.get_unchecked(current_index))
    }
}
