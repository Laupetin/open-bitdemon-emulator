use crate::messaging::bd_serialization::BdSerialize;

#[derive(Clone)]
pub struct ResultSlice<T> {
    data: Vec<T>,
    offset: usize,
    total_count: Option<usize>,
}

impl<T: 'static> ResultSlice<T> {
    pub fn new(data: Vec<T>, offset: usize) -> Self {
        ResultSlice {
            data,
            offset,
            total_count: None,
        }
    }

    pub fn with_total_count(data: Vec<T>, offset: usize, total_count: usize) -> Self {
        ResultSlice {
            data,
            offset,
            total_count: Some(total_count),
        }
    }

    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    pub fn into_data(self) -> Vec<T> {
        self.data
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn total_count(&self) -> usize {
        self.total_count.unwrap_or_else(|| self.data.len())
    }

    pub fn boxed<T2: From<T>>(self) -> ResultSlice<Box<T2>>
    where
        Vec<Box<T2>>: FromIterator<Box<T>>,
    {
        let offset = self.offset;
        let total_count = self.total_count;
        let data = self.data.into_iter().map(|el| Box::from(el)).collect();

        ResultSlice {
            data,
            offset,
            total_count,
        }
    }

    pub fn serializable(self) -> ResultSlice<Box<dyn BdSerialize>>
    where
        T: BdSerialize,
    {
        let offset = self.offset;
        let total_count = self.total_count;
        let data = self
            .data
            .into_iter()
            .map(|el| Box::from(el) as Box<dyn BdSerialize>)
            .collect();

        ResultSlice {
            data,
            offset,
            total_count,
        }
    }
}
