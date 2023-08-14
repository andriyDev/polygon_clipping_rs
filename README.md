# polygon_clipping_rs

A Rust crate to compute boolean operations (i.e., intersection, union,
difference, and XOR) of two polygons.

## Polygon representation

Polygons are represented as a set of "contours". Each contour is a loop of
vertices. Points contained by an even number of contours are considered outside
the polygon, and points contained by an odd number of contours are considered
inside the polygon. Note that "polygon" is not quite correct since this includes
"multipolygons" - essentially two completely disjoint shapes.

### Invalid/malformed polygons

This implementation does not account for "malformed" polygons. The behavior in
these cases is undefined. Some malformed polygons include:

* Polygons containing `NaN` or `Infinity` coordinates. It is pretty obvious why
this would be a problem.
* Polygons containing overlapping edges. If a single polygon contains
overlapping edges, it is unclear what the edge implies. In other words, any
polygon with overlapping edges can be "reorganized" such that the overlapping
edges are not present in the new polygon - the overlapping edge was never
needed!

## Algorithm

This is an implementation of the paper:
```
Francisco Martínez, Carlos Ogayar, Juan R. Jiménez, Antonio J. Rueda,
A simple algorithm for Boolean operations on polygons,
Advances in Engineering Software,
Volume 64,
2013,
Pages 11-19,
ISSN 0965-9978,
https://doi.org/10.1016/j.advengsoft.2013.04.004.
(https://www.sciencedirect.com/science/article/pii/S0965997813000379)
```

### Differences to the original algorithm

These are intentional changes to the original algorithm.

* The paper reports using pointers for everything. This makes cleanup messy and
Rust really doesn't like all the cyclic references for obvious reasons. This
implementation uses separate data structures to split the data into chunks we
can mutate independently. This can mean there is more (maybe less though)
memory usage than the paper's implementation. However, this implementation uses
fewer small allocations - allocations are batched together.
* In addition to the result polygon, we also return the source edges for each
edge in the result polygon. This is useful when there is some "metadata" about
edges in the source polygons that you would like to retain in the result
polygon. An example is if each polygon is a room with edges being walls, but
some edges are doors. It may be useful to know which edges in the result polygon
are still doors.

### Deficiencies to the original algorithm

These are problems in the implementation that could be addressed in the future.

* The paper describes using a binary search tree for the "sweep line"
data structure. The current implementation uses a sorted `Vec`, so some
operations may have different performance characteristics. The paper also
mentions that events could store their position in the sweep line to avoid a
search.
* This implementation does not properly handle more than two edges of a contour
meeting at a single vertex. The paper briefly mentions a solution (although not
as clear as I would like).

### A personal note

This paper is quite clever, and the general idea is fairly intuitive.
**However**, the paper seems to hide critical information in single sentences
that seem benign (e.g., events must be sorted by non-vertical edges first), and
more importantly leaves a lot of figuring out the details of the algorithm to
the reader. This made it confusing when my version required significant
differences to the pseudo-code in the paper. In addition, some flags are
described but not how to compute them, and the "special cases" are treated as
footnotes rather than parts of the algorithm that take lots of time and work to
restructure and figure out.

Alright, I'm done complaining.

## License

Licensed under the [MIT license](LICENSE).
