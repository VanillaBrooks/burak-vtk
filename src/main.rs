mod cli;

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::Deserialize;

use ndarray::{Array3, Array4};

#[derive(Deserialize)]
/// column headers of the CSV file
struct CsvData {
    x: f64,
    y: f64,
    z: f64,
    u1r: f64,
    u2r: f64,
    u3r: f64,
    u1i: f64,
    u2i: f64,
    u3i: f64,
    w1r: f64,
    w2r: f64,
    w3r: f64,
    w1i: f64,
    w2i: f64,
    w3i: f64,
}

#[derive(vtk::DataArray)]
/// VTK/VTR output data for paraview
struct VtkData {
    real_velocity: vtk::Vector3D<f64>,
    imaginary_velocity: vtk::Vector3D<f64>,
    total_velocity_magnitude: vtk::Scalar3D<f64>,
    real_w: vtk::Vector3D<f64>,
    imaginary_w: vtk::Vector3D<f64>,
    total_w_magnitude: vtk::Scalar3D<f64>,
}

impl VtkData {
    fn new(
        real_velocity: ndarray::Array4<f64>,
        imaginary_velocity: ndarray::Array4<f64>,
        total_velocity_magnitude: ndarray::Array3<f64>,
        real_w: ndarray::Array4<f64>,
        imaginary_w: ndarray::Array4<f64>,
        total_w_magnitude: ndarray::Array3<f64>,
    ) -> Self {
        Self {
            real_velocity: vtk::Vector3D::new(real_velocity),
            imaginary_velocity: vtk::Vector3D::new(imaginary_velocity),
            total_velocity_magnitude: vtk::Scalar3D::new(total_velocity_magnitude),
            real_w: vtk::Vector3D::new(real_w),
            imaginary_w: vtk::Vector3D::new(imaginary_w),
            total_w_magnitude: vtk::Scalar3D::new(total_w_magnitude),
        }
    }
}

fn magnitude_complex(
    nx: usize,
    ny: usize,
    nz: usize,
    real: &Array4<f64>,
    im: &Array4<f64>,
    out: &mut Array3<f64>,
) {
    for i in 0..nx {
        for j in 0..ny {
            for k in 0..nz {
                let mut magnitude_squared = 0.;

                // the magnitude of a vector of complex numbers is the sum of the squares of all
                // components
                for v in 0..3 {
                    magnitude_squared += real[[v, i, j, k]].powi(2);
                    magnitude_squared += im[[v, i, j, k]].powi(2);
                }

                out[[i, j, k]] = magnitude_squared.sqrt();
            }
        }
    }
}

fn determine_spans(file: std::fs::File) -> Result<(vtk::Spans3D, vtk::Mesh3D<f64, vtk::Binary>)> {
    let reader = std::io::BufReader::new(file);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut z = Vec::new();

    for (idx, row) in reader.deserialize().enumerate() {
        let row: CsvData =
            row.with_context(|| format!("failed to serialize row {} of csv", idx + 2))?;

        if !x.contains(&row.x) {
            x.push(row.x)
        }
        if !y.contains(&row.y) {
            y.push(row.y)
        }
        if !z.contains(&row.z) {
            z.push(row.z)
        }
    }

    let spans = vtk::Spans3D::new(x.len(), y.len(), z.len());
    let mesh = vtk::Mesh3D::<f64, vtk::Binary>::new(x, y, z);
    Ok((spans, mesh))
}

fn main() -> Result<()> {
    let args = cli::Args::parse();

    let file = std::fs::File::open(&args.csv_path)
        .with_context(|| format!("failed to open CSV file at {}", args.csv_path.display()))?;

    let (spans, mesh) = determine_spans(file)
        .with_context(|| format!("failed to read span and mesh information from CSV {} on initial pass", args.csv_path.display()))?;

    let nx = spans.x_len();
    let ny = spans.y_len();
    let nz = spans.z_len();

    println!("mesh size is ({nx},{ny},{nz})");

    // now, re-open the file to refresh the reader
    let file = std::fs::File::open(&args.csv_path)
        .with_context(|| format!("failed to open CSV file at {}", args.csv_path.display()))?;
    let reader = std::io::BufReader::new(file);

    // open the writer
    let writer = std::fs::File::create(&args.output)
        .with_context(|| format!("failed to create output file at {}", args.output.display()))?;
    let writer = std::io::BufWriter::new(writer);

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(reader);

    let mut iter = reader.deserialize().enumerate();


    let mut real_velocity: Array4<f64> = Array4::zeros((3, nx, ny, nz));
    let mut imaginary_velocity: Array4<f64> = Array4::zeros((3, nx, ny, nz));
    let mut total_velocity_magnitude: Array3<f64> = Array3::zeros((nx, ny, nz));
    let mut real_w: Array4<f64> = Array4::zeros((3, nx, ny, nz));
    let mut imaginary_w: Array4<f64> = Array4::zeros((3, nx, ny, nz));
    let mut total_w_magnitude: Array3<f64> = Array3::zeros((nx, ny, nz));

    for i in 0..nx {
        for j in 0..ny {
            for k in 0..nz {
                // read the next row in the CSV, error if the row does not exist
                let (idx, row) = if let Some(row) = iter.next() {
                    row
                } else {
                    bail!("CSV was shorter than expected, unable to find data point for ({i},{j},{k}) - the wrong value of `--n` may have been chosen");
                };

                let row: CsvData =
                    row.with_context(|| format!("failed to serialize row {} of csv", idx + 2))?;


                //
                // pull velocity information into containers
                //

                real_velocity[[0, i, j, k]] = row.u1r;
                real_velocity[[1, i, j, k]] = row.u2r;
                real_velocity[[2, i, j, k]] = row.u3r;

                imaginary_velocity[[0, i, j, k]] = row.u1i;
                imaginary_velocity[[1, i, j, k]] = row.u2i;
                imaginary_velocity[[2, i, j, k]] = row.u3i;

                real_w[[0, i, j, k]] = row.w1r;
                real_w[[1, i, j, k]] = row.w2r;
                real_w[[2, i, j, k]] = row.w3r;

                imaginary_w[[0, i, j, k]] = row.w1i;
                imaginary_w[[1, i, j, k]] = row.w2i;
                imaginary_w[[2, i, j, k]] = row.w3i;
            }
        }
    }

    if let Some(_) = iter.next() {
        bail!("unread data in csv - this should not happen");
    }

    magnitude_complex(
        nx,
        ny,
        nz,
        &real_velocity,
        &imaginary_velocity,
        &mut total_velocity_magnitude,
    );
    magnitude_complex(
        nx,
        ny,
        nz,
        &real_w,
        &imaginary_w,
        &mut total_w_magnitude,
    );

    let data = VtkData::new(
        real_velocity,
        imaginary_velocity,
        total_velocity_magnitude,
        real_w,
        imaginary_w,
        total_w_magnitude,
    );


    let domain = vtk::Rectilinear3D::new(mesh, spans);
    let vtk_write = vtk::VtkData::new(domain, data);

    vtk::write_vtk(writer, vtk_write).with_context(|| "failed to write final vtk file")?;

    Ok(())
}
