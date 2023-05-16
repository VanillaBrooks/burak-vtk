# burak-vtk

Helper utility to convert CSV files to VTK files with vector layouts. This project is specialized
for a specific problem and probably not relevant to you


## Usage


within the `burak-vtk` directory of  this project, with `cargo` installed:

```
cargo r --release --  --csv-path path/to/your/file.csv --output path/to/output/file.vtr
```

the `.vtr` extension specifies to paraview to a rectilinear coordinate reader on the file.


## Example

if `case.csv` is in your local directory:

```
cargo r --release --  --csv-path case.csv --output case.vtr
```
