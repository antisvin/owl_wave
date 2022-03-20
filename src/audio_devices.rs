use std::collections::HashMap;

use cpal::{
    traits::HostTrait,
    traits::{DeviceTrait, StreamTrait},
    Device, Host, HostId, Sample, SampleFormat, Stream,
};

pub struct AudioHandler {
    pub hosts: HashMap<HostId, Host>,
    pub input_devices: HashMap<HostId, Vec<Device>>,
    pub output_devices: HashMap<HostId, Vec<Device>>,
    pub audio_loaded: bool,
    pub output_stream: Option<Stream>,
}

impl AudioHandler {
    pub fn new() -> Self {
        AudioHandler {
            hosts: HashMap::<HostId, Host>::new(),
            input_devices: HashMap::<HostId, Vec<Device>>::new(),
            output_devices: HashMap::<HostId, Vec<Device>>::new(),
            audio_loaded: false,
            output_stream: None,
        }
    }
    pub fn scan(&mut self) -> &mut Self {
        self.hosts.clear();
        self.output_devices.clear();
        self.input_devices.clear();
        for host_id in cpal::available_hosts() {
            if let Ok(host) = cpal::host_from_id(host_id) {
                if let Ok(input_devices) = host.input_devices() {
                    for (i, device) in input_devices.enumerate() {
                        if i == 0 {
                            self.input_devices.insert(host_id, Vec::<Device>::new());
                        }
                        #[cfg(feature = "jack")]
                        if host_id == cpal::HostId::Jack
                            && device.name().unwrap() == "cpal_client_out"
                        {
                            continue;
                        }
                        self.input_devices.get_mut(&host_id).unwrap().push(device);
                    }
                }
                if let Ok(output_devices) = host.output_devices() {
                    for (i, device) in output_devices.enumerate() {
                        if i == 0 {
                            self.output_devices.insert(host_id, Vec::<Device>::new());
                        }
                        #[cfg(feature = "jack")]
                        if host_id == cpal::HostId::Jack
                            && device.name().unwrap() == "cpal_client_in"
                        {
                            continue;
                        }
                        self.output_devices.get_mut(&host_id).unwrap().push(device);
                    }
                }
                self.hosts.insert(host_id, host);
            }
        }
        self.audio_loaded = true;
        self
    }
    pub fn select_output(&mut self, maybe_host_id: Option<HostId>, maybe_device_id: Option<usize>) {
        if let Some(host_id) = maybe_host_id {
            if let Some(device_id) = maybe_device_id {
                let err_fn = |err| print!("an error occurred on the output audio stream: {}", err);
                let device = self
                    .output_devices
                    .get(&host_id)
                    .unwrap()
                    .get(device_id)
                    .unwrap();
                let mut supported_configs_range = device
                    .supported_output_configs()
                    .expect("error while querying configs");
                let supported_config = supported_configs_range
                    .next()
                    .expect("no supported config?!")
                    .with_max_sample_rate();
                let sample_format = supported_config.sample_format();
                let config = supported_config.into();
                self.output_stream = match sample_format {
                    SampleFormat::F32 => {
                        device.build_output_stream(&config, write_silence::<f32>, err_fn)
                    }
                    SampleFormat::I16 => {
                        device.build_output_stream(&config, write_silence::<i16>, err_fn)
                    }
                    SampleFormat::U16 => {
                        device.build_output_stream(&config, write_silence::<u16>, err_fn)
                    }
                }
                .ok();
                if let Some(stream) = &self.output_stream {
                    stream.play().unwrap()
                }
                /*
                self.output_stream = device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            // react to stream events and read or write stream data here.
                            write_data(data, channels, &mut next_value)
                        },
                        move |_err| {
                            // react to errors here.
                        },
                    )
                    .ok();
                     */
            }
        }
    }

    pub fn select_input(&mut self, _host_id: Option<HostId>, _device: Option<usize>) {}
}
/*
fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
 */

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::from(&0.0);
    }
}
/*
use cpal::traits::DeviceTrait;
use cpal::{Sample, SampleFormat, Stream};

use cpal::traits::StreamTrait;

pub struct Handle(Stream);

pub fn init_audio() -> Handle {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("failed to find a default output device");
    let config = device.default_output_config().unwrap();

    Handle(match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    })
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Stream
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| print!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_data(data, channels, &mut next_value),
            err_fn,
        )
        .unwrap();
    stream.play().unwrap();
    stream
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
*/
