use super::metrics::{ChunkSummary, SummaryUpTo, ToByteOffset};
use super::utils::{debug_no_quotes, panic_messages as panic};
use crate::tree::{Metric, Summarize};

/// A slice of a [`GapBuffer`](super::gap_buffer::GapBuffer).
#[derive(Copy, Clone, Default)]
pub struct GapSlice<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) left_summary: ChunkSummary,
    pub(super) len_right: u16,
}

impl core::fmt::Debug for GapSlice<'_> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("\"")?;
        debug_no_quotes(self.left_chunk(), f)?;
        write!(f, "{:~^1$}", "", self.len_gap())?;
        debug_no_quotes(self.right_chunk(), f)?;
        f.write_str("\"")
    }
}

// We only need this to compare `GapSlice`s with `&str`s in (doc)tests.
impl PartialEq<GapSlice<'_>> for &str {
    fn eq(&self, rhs: &GapSlice<'_>) -> bool {
        self.len() == rhs.len()
            && rhs.left_chunk() == &self[..rhs.len_left()]
            && rhs.right_chunk() == &self[rhs.len_left()..]
    }
}

impl<'a> GapSlice<'a> {
    /// Panics with a nicely formatted error message if the given byte offset
    /// is not a character boundary.
    #[track_caller]
    #[inline]
    pub(super) fn assert_char_boundary(&self, byte_offset: usize) {
        debug_assert!(byte_offset <= self.len());

        if !self.is_char_boundary(byte_offset) {
            if byte_offset < self.len_left() {
                panic::byte_offset_not_char_boundary(
                    self.left_chunk(),
                    byte_offset,
                )
            } else {
                panic::byte_offset_not_char_boundary(
                    self.right_chunk(),
                    byte_offset - self.len_left(),
                )
            }
        }
    }

    pub(super) fn assert_invariants(&self) {
        assert_eq!(self.left_summary, ChunkSummary::from(self.left_chunk()));

        if self.len_right() == 0 {
            assert_eq!(self.len_left(), self.bytes.len());
        } else if self.len_left() == 0 {
            assert_eq!(self.len_right(), self.bytes.len());
        }
    }

    /// Returns the byte at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds, i.e. greater than or equal to
    /// [`len()`](Self::len()).
    #[inline]
    pub(super) fn byte(&self, byte_index: usize) -> u8 {
        debug_assert!(byte_index < self.len());

        if byte_index < self.len_left() {
            self.left_chunk().as_bytes()[byte_index]
        } else {
            self.right_chunk().as_bytes()[byte_index - self.len_left()]
        }
    }

    #[inline]
    fn left_measure<M>(&self) -> M
    where
        M: Metric<ChunkSummary>,
    {
        M::measure(&self.left_summary)
    }

    #[inline]
    pub(super) fn truncate_last_char(
        &mut self,
        summary: ChunkSummary,
    ) -> ChunkSummary {
        debug_assert!(self.len() > 0);
        debug_assert_eq!(summary, self.summarize());

        use core::cmp::Ordering;

        let last_char = self
            .last_chunk()
            .chars()
            .next_back()
            .expect("this slice isn't empty");

        let removed_summary = ChunkSummary::from(last_char);

        let len_utf8 = removed_summary.bytes();

        match self.len_right.cmp(&(len_utf8 as u16)) {
            // The slice doesn't have a right chunk, so we shorten the left
            // chunk.
            Ordering::Less => {
                self.left_summary -= removed_summary;
                self.bytes = &self.bytes[..self.len_left()];
                self.left_summary
            },

            // The right chunk has 2 or more characters, so we shorten the right
            // chunk.
            Ordering::Greater => {
                self.len_right -= len_utf8 as u16;
                self.bytes = &self.bytes[..self.bytes.len() - len_utf8];
                summary - removed_summary
            },

            // The right chunk has exactly 1 character, so we can keep just the
            // left chunk.
            Ordering::Equal => {
                self.len_right = 0;
                self.bytes = &self.bytes[..self.len_left()];
                self.left_summary
            },
        }
    }

    /// Removes the trailing line break (if it has one), returning the new
    /// summary.
    #[inline]
    pub(super) fn truncate_trailing_line_break(
        &mut self,
        summary: ChunkSummary,
    ) -> ChunkSummary {
        debug_assert_eq!(summary, self.summarize());

        if !self.has_trailing_newline() {
            return summary;
        }

        let mut new_summary = self.truncate_last_char(summary);

        if self.last_chunk().ends_with('\r') {
            new_summary = self.truncate_last_char(new_summary)
        }

        new_summary
    }

    #[inline]
    pub(super) fn empty() -> Self {
        Self::default()
    }

    /// Returns `true` if it ends with a newline.
    #[inline]
    pub(super) fn has_trailing_newline(&self) -> bool {
        self.last_chunk().ends_with('\n')
    }

    #[inline]
    pub(super) fn is_char_boundary(&self, byte_offset: usize) -> bool {
        debug_assert!(byte_offset <= self.len());

        if byte_offset <= self.len_left() {
            self.left_chunk().is_char_boundary(byte_offset)
        } else {
            self.right_chunk().is_char_boundary(byte_offset - self.len_left())
        }
    }

    /// The second segment if it's not empty, or the first one otherwise.
    #[inline]
    pub(super) fn last_chunk(&self) -> &'a str {
        if self.len_right() == 0 {
            self.left_chunk()
        } else {
            self.right_chunk()
        }
    }

    #[inline]
    pub(super) fn left_chunk(&self) -> &'a str {
        // SAFETY: the first `len_left` bytes are valid UTF-8.
        unsafe {
            core::str::from_utf8_unchecked(&self.bytes[..self.len_left()])
        }
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.len_left() + self.len_right()
    }

    #[inline]
    pub(super) fn len_gap(&self) -> usize {
        self.bytes.len() - self.len()
    }

    #[inline]
    pub(super) fn len_left(&self) -> usize {
        self.left_summary.bytes()
    }

    #[inline]
    pub(super) fn len_right(&self) -> usize {
        self.len_right as _
    }

    #[inline]
    pub(super) fn right_chunk(&self) -> &'a str {
        // SAFETY: the last `len_right` bytes are valid UTF-8.
        unsafe {
            core::str::from_utf8_unchecked(
                &self.bytes[self.bytes.len() - self.len_right()..],
            )
        }
    }

    #[inline]
    fn right_summary(&self, summary: ChunkSummary) -> ChunkSummary {
        debug_assert_eq!(summary, self.summarize());
        summary - self.left_summary
    }

    /// Splits the slice at the given offset, returning the left and right
    /// slices and their summary.
    ///
    /// # Panics
    ///
    /// Panics if the offset is greater than the M-measure of the slice.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let gap_buffer = GapBuffer::<20>::from("foo\nbar\r\nbaz");
    ///
    /// let summary = gap_buffer.summarize();
    ///
    /// let ((left, _), (right, _)) =
    ///     gap_buffer.as_slice().split_at_offset(RawLineMetric(1));
    ///
    /// assert_eq!("foo\n", left);
    ///
    /// assert_eq!("bar\r\nbaz", right);
    /// ```
    #[track_caller]
    #[inline]
    pub fn split_at_offset<M>(
        &self,
        mut offset: M,
        summary: ChunkSummary,
    ) -> ((Self, ChunkSummary), (Self, ChunkSummary))
    where
        M: Metric<ChunkSummary> + ToByteOffset + SummaryUpTo,
    {
        debug_assert_eq!(summary, self.summarize());

        debug_assert!(offset <= M::measure(&summary));

        if offset <= self.left_measure::<M>() {
            let byte_offset: usize = offset.to_byte_offset(self.left_chunk());

            let (bytes_left, bytes_right) = self.split_bytes(byte_offset);

            let left_left_summary = M::up_to(
                self.left_chunk(),
                self.left_summary,
                offset,
                byte_offset,
            );

            let left = Self {
                bytes: bytes_left,
                left_summary: left_left_summary,
                len_right: 0,
            };

            let right = Self {
                bytes: bytes_right,
                left_summary: self.left_summary - left_left_summary,
                len_right: self.len_right,
            };

            ((left, left.left_summary), (right, summary - left.left_summary))
        } else {
            offset -= self.left_measure::<M>();

            let byte_offset = offset.to_byte_offset(self.right_chunk());

            let (bytes_left, bytes_right) =
                self.split_bytes(self.len_left() + byte_offset);

            let right_left_summary = M::up_to(
                self.right_chunk(),
                self.right_summary(summary),
                offset,
                byte_offset,
            );

            let left = Self {
                bytes: bytes_left,
                left_summary: self.left_summary,
                len_right: right_left_summary.bytes() as u16,
            };

            let right = Self {
                bytes: bytes_right,
                left_summary: self.right_summary(summary) - right_left_summary,
                len_right: 0,
            };

            ((left, summary - right.left_summary), (right, right.left_summary))
        }
    }

    #[inline]
    fn split_bytes(&self, byte_offset: usize) -> (&'a [u8], &'a [u8]) {
        debug_assert!(byte_offset <= self.len());

        use core::cmp::Ordering;

        let offset = match byte_offset.cmp(&self.len_left()) {
            Ordering::Less => byte_offset,

            Ordering::Greater => byte_offset + self.len_gap(),

            Ordering::Equal => {
                return (
                    self.left_chunk().as_bytes(),
                    self.right_chunk().as_bytes(),
                );
            },
        };

        self.bytes.split_at(offset)
    }

    #[inline]
    fn summarize_right_chunk(&self) -> ChunkSummary {
        ChunkSummary::from(self.right_chunk())
    }
}

impl Summarize for GapSlice<'_> {
    type Summary = ChunkSummary;

    #[inline]
    fn summarize(&self) -> Self::Summary {
        self.left_summary + self.summarize_right_chunk()
    }
}

#[cfg(test)]
mod tests {
    use crate::rope::gap_buffer::GapBuffer;
    use crate::tree::{AsSlice, Summarize};

    #[test]
    fn debug_slice() {
        let buffer = GapBuffer::<10>::from("Hello");
        assert_eq!("\"He~~~~~llo\"", format!("{:?}", buffer.as_slice()));
    }

    #[test]
    fn truncate_trailing_crlf() {
        let buffer = GapBuffer::<5>::from("bar\r\n");
        let mut slice = buffer.as_slice();
        let summary = slice.summarize();
        slice.truncate_trailing_line_break(summary);
        assert_eq!("bar", slice);
    }

    #[test]
    fn truncate_trailing_lf() {
        let buffer = GapBuffer::<5>::from("bar\n");
        let mut slice = buffer.as_slice();
        let summary = slice.summarize();
        slice.truncate_trailing_line_break(summary);
        assert_eq!("bar", slice);
    }
}
