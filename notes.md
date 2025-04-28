If the last line is empty (e.g. trailing newline),
e.g. foo\n
iterator should have units_total = 2 but if bytes_left = 0 then yields empty slice

otherwise,
it can yield slice as normal
INITIALIZING BACK ByteMetric(4)
calling next back
curr iter back UnitsBackward { is_initialized: false, path: [], leaf_node: Leaf(Lnode { value: "fo[GAP 2044]o\n", summary: ChunkSummary { bytes: 4, line_breaks: 1 } }), yielded_in_leaf: ChunkSummary { bytes: 0, line_breaks: 0 }, end_slice: "[GAP 0]", end_summary: ChunkSummary { bytes: 0, line_breaks: 0 }, first_slice: None, last_slice: None, base_start: ByteMetric(0), base_remaining: ByteMetric(4), units_remaining: LineMetric(2) }
split into "fo[GAP 2044]o\n" ChunkSummary { bytes: 4, line_breaks: 1 }
and "[GAP 0]" ChunkSummary { bytes: 0, line_breaks: 0 }
split into "fo[GAP 2044]o\n" ChunkSummary { bytes: 4, line_breaks: 1 }
and "[GAP 0]" ChunkSummary { bytes: 0, line_breaks: 0 }

INITIALIZING BACK ByteMetric(7)
calling next back
curr iter back UnitsBackward { is_initialized: false, path: [], leaf_node: Leaf(Lnode { value: "foo[GAP 2041]\nbar", summary: ChunkSummary { bytes: 7, line_breaks: 1 } }), yielded_in_leaf: ChunkSummary { bytes: 0, line_breaks: 0 }, end_slice: "[GAP 0]", end_summary: ChunkSummary { bytes: 0, line_breaks: 0 }, first_slice: None, last_slice: None, base_start: ByteMetric(0), base_remaining: ByteMetric(7), units_remaining: LineMetric(2) }
split into "foo[GAP 2041]\n" ChunkSummary { bytes: 4, line_breaks: 1 }
and "bar[GAP 0]" ChunkSummary { bytes: 3, line_breaks: 0 }
calling next back
curr iter back UnitsBackward { is_initialized: true, path: [], leaf_node: Leaf(Lnode { value: "foo[GAP 2041]\nbar", summary: ChunkSummary { bytes: 7, line_breaks: 1 } }), yielded_in_leaf: ChunkSummary { bytes: 3, line_breaks: 0 }, end_slice: "foo[GAP 2041]\n", end_summary: ChunkSummary { bytes: 4, line_breaks: 1 }, first_slice: None, last_slice: None, base_start: ByteMetric(0), base_remaining: ByteMetric(4), units_remaining: LineMetric(1) }
