#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();
}

use std::sync::Arc;

use anyhow::{Result, anyhow};
use burn::{
    Tensor,
    tensor::{Bytes, TensorCreationOptions, TensorData},
};
use colorgrad::Gradient;
use image::DynamicImage;
use jiff::ToSpan;
use opendal::{Operator, services::Gcs};
use zarrs::{
    array::{Array, data_type::Float32DataType},
    storage::{AsyncReadableListableStorage, AsyncReadableListableStorageTraits},
};
use zarrs_opendal::AsyncOpendalStore;

fn norm<const D: usize>(t: Tensor<D>) -> Tensor<D> {
    let min: f32 = 210.0;
    let max: f32 = 310.0;

    t.sub_scalar(min).div_scalar(max - min)
}

fn zarr_tensor_to_burn_tensor<const D: usize>(tensor: zarrs::array::Tensor) -> Result<Tensor<D>> {
    let (zarr_bytes, zarr_dtype, zarr_shape) = tensor.into_parts();

    let burn_dtype: burn::tensor::DType = match zarr_dtype.is::<Float32DataType>() {
        true => Ok(burn::tensor::DType::F32),
        false => Ok(burn::tensor::DType::I64),
        _ => Err(anyhow!("Non matching type found")),
    }?;

    let burn_bytes = Bytes::from_bytes_vec(zarr_bytes.to_vec());

    let tensor_data = TensorData::try_from_bytes(burn_bytes, zarr_shape, burn_dtype)?;
    let tensor = Tensor::<D>::from_data(tensor_data, TensorCreationOptions::default());

    Ok(tensor)
}

async fn get_lat_lon(
    store: Arc<impl AsyncReadableListableStorageTraits + 'static + ?Sized>,
) -> Result<(Tensor<1>, Tensor<1>, Tensor<1>)> {
    let component = "time";
    let time_arr = Array::async_open(store.clone(), &format!("/{}", component)).await?;

    let end = time_arr.shape();
    let time: zarrs::array::Tensor = time_arr.async_retrieve_array_subset(&[0..end[0]]).await?;

    let component = "latitude";
    let lat_arr = Array::async_open(store.clone(), &format!("/{}", component)).await?;

    let end = lat_arr.shape();
    let lat: zarrs::array::Tensor = lat_arr.async_retrieve_array_subset(&[0..end[0]]).await?;

    let component = "longitude";
    let lon_arr = Array::async_open(store.clone(), &format!("/{}", component)).await?;

    let end = lon_arr.shape();
    let lon: zarrs::array::Tensor = lon_arr.async_retrieve_array_subset(&[0..end[0]]).await?;

    let component = "level";
    let level_arr = Array::async_open(store.clone(), &format!("/{}", component)).await?;
    println!("{:#?}", level_arr.metadata());

    let end = level_arr.shape();
    let level: zarrs::array::Tensor = level_arr.async_retrieve_array_subset(&[0..end[0]]).await?;
    let level_tensor: Tensor<1> = zarr_tensor_to_burn_tensor(level)?;
    println!("{}", level_tensor);

    let time_tensor = zarr_tensor_to_burn_tensor(time)?;
    let lat_tensor = zarr_tensor_to_burn_tensor(lat)?;
    let lon_tensor = zarr_tensor_to_burn_tensor(lon)?;

    Ok((time_tensor, lat_tensor, lon_tensor))
}

#[tokio::main]
pub async fn gen_texture() -> Result<()> {
    let builder = Gcs::default()
        .bucket("gcp-public-data-arco-era5")
        .root("/ar/full_37-1h-0p25deg-chunk-1.zarr-v3")
        .skip_signature();

    let operator = Operator::new(builder)?;

    let store: AsyncReadableListableStorage = Arc::new(AsyncOpendalStore::new(operator));

    let component = "temperature";
    let array = Array::async_open(store.clone(), &format!("/{}", component))
        .await?
        .to_v3()?;
    println!("{:#?}", array.metadata());

    let (time, lat, lon) = get_lat_lon(store.clone()).await?;

    println!("{}", time);
    println!("{}", lat);
    println!("{}", lon);

    let zarr_tensor: zarrs::array::Tensor = array
        .async_retrieve_array_subset(&[1_000_004..1_000_005, 36..37, 0..721, 0..1440])
        .await?;

    let (zarr_bytes, zarr_dtype, zarr_shape) = zarr_tensor.into_parts();
    let burn_dtype: burn::tensor::DType = match zarr_dtype.downcast_ref() {
        Some(Float32DataType) => Ok(burn::tensor::DType::F32),
        _ => Err(anyhow!("Non matching type found")),
    }?;

    let burn_bytes = Bytes::from_bytes_vec(zarr_bytes.to_vec());

    let tensor_data = TensorData::try_from_bytes(burn_bytes, zarr_shape, burn_dtype)?;
    let tensor = Tensor::<4>::from_data(tensor_data, TensorCreationOptions::default());
    let mut tensor = tensor.reshape([721, 1440]);

    tensor.inplace(norm);
    let grad = colorgrad::preset::viridis();

    let chunks: Vec<f32> = tensor.t().clone().try_into_data()?.try_into_vec()?;
    let t: f32 = time.try_into_data()?.try_to_vec()?[1_000_004];

    println!("t: {}", t);

    let timestap = jiff::civil::date(1900, 01, 01).at(0, 0, 0, 0);
    println!("t: {:#?}", timestap.checked_add(1_000_004.hours())?);

    let mut imgbuf = DynamicImage::new_rgba32f(1440, 721).into_rgba32f();

    for (x, chunk) in chunks.chunks(721).enumerate() {
        for (y, val) in chunk.iter().enumerate() {
            imgbuf.put_pixel(x as u32, y as u32, image::Rgba(grad.at(*val).to_a98_rgb()));
        }
    }

    imgbuf.save(format!("{}3.tiff", component))?;
    Ok(())
}
