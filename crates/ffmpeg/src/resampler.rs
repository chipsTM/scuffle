use rusty_ffmpeg::ffi::{swr_alloc_set_opts2, swr_convert_frame, swr_free, swr_init, SwrContext};

use crate::{
    enums::AVSampleFormat,
    error::{FfmpegError, FfmpegErrorCode},
    frame::{AudioChannelLayout, AudioFrame, GenericFrame},
    smart_object::SmartPtr,
};

/// A wrapper around an [`SwrContext`]. Which is used to resample and convert [`AudioFrame`]s.
pub struct Resampler {
    ptr: SmartPtr<SwrContext>,
    channel_layout: AudioChannelLayout,
    sample_fmt: AVSampleFormat,
    sample_rate: i32,
}

impl Resampler {
    /// Create a new [`Resampler`] instance
    pub fn new(
        input_ch_layout: AudioChannelLayout,
        input_sample_fmt: AVSampleFormat,
        input_sample_rate: i32,
        output_ch_layout: AudioChannelLayout,
        output_sample_fmt: AVSampleFormat,
        output_sample_rate: i32,
    ) -> Result<Self, FfmpegError> {
        let mut ptr = core::ptr::null_mut::<SwrContext>();

        // Safety: swr_alloc_set_opts2 is safe to call
        FfmpegErrorCode(unsafe {
            swr_alloc_set_opts2(
                &mut ptr,
                output_ch_layout.as_ptr(),
                output_sample_fmt.into(),
                output_sample_rate,
                input_ch_layout.as_ptr(),
                input_sample_fmt.into(),
                input_sample_rate,
                0,
                core::ptr::null::<core::ffi::c_void>() as _,
            )
        })
        .result()?;

        let destructor = |ctx: &mut *mut SwrContext| {
            // Safety: swr_free is safe to call
            unsafe { swr_free(ctx) };
        };

        // Safety: this is safe to call
        let mut ptr = unsafe { SmartPtr::wrap_non_null(ptr, destructor).ok_or(FfmpegError::Alloc) }?;

        // Safety: ptr is initialized, swr_init is safe to call
        FfmpegErrorCode(unsafe { swr_init(ptr.as_mut_ptr()) }).result()?;

        Ok(Self {
            ptr,
            channel_layout: output_ch_layout,
            sample_fmt: output_sample_fmt,
            sample_rate: output_sample_rate,
        })
    }

    /// Process an [`AudioFrame`] thought the resampler
    pub fn process(&mut self, input: &AudioFrame) -> Result<AudioFrame, FfmpegError> {
        let mut out = GenericFrame::new()?;

        // Safety: the GenericFrame is allocated
        let inner = unsafe { out.as_mut_ptr().as_mut() }.expect("inner pointer of GenericFrame was invalid");
        inner.ch_layout = self.channel_layout().copy()?.into_inner();
        inner.format = self.sample_format().into();
        inner.sample_rate = self.sample_rate();

        // Safety: self.ptr is initialized and valid, data buffers of out get initialized here, swr_convert_frame is safe to call
        FfmpegErrorCode(unsafe { swr_convert_frame(self.ptr.as_mut_ptr(), out.as_mut_ptr(), input.as_ptr()) }).result()?;

        // Safety: swr_convert_frame was successful, the pointer is valid;
        Ok(out.audio())
    }

    /// The output channel layout
    pub const fn channel_layout(&self) -> &AudioChannelLayout {
        &self.channel_layout
    }

    /// The output sample format
    pub const fn sample_format(&self) -> AVSampleFormat {
        self.sample_fmt
    }

    /// The output sample rate
    pub const fn sample_rate(&self) -> i32 {
        self.sample_rate
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use rand::{rng, Rng};
    use rusty_ffmpeg::ffi::swr_is_initialized;

    use crate::{
        frame::{AudioChannelLayout, AudioFrame},
        AVSampleFormat,
    };

    use super::Resampler;

    #[test]
    fn test_resampler_new() {
        let input_layout = AudioChannelLayout::new(1).expect("Failed to create new AudioChannelLayout");
        let input_format = AVSampleFormat::S16;
        let input_sample_rate = 44100;

        let output_layout = AudioChannelLayout::new(2).expect("Failed to create new AudioChannelLayout");
        let output_format = AVSampleFormat::S16p;
        let output_sample_rate = 48000;

        let mut resampler = Resampler::new(
            input_layout,
            input_format,
            input_sample_rate,
            output_layout,
            output_format,
            output_sample_rate,
        )
        .expect("Failed to create new Resampler");

        // Safety: swr_is_initialized is safe to call
        let is_init = unsafe { swr_is_initialized(resampler.ptr.as_mut_ptr()) };

        assert!(
            is_init.is_positive() && is_init != 0,
            "Resampler is not initialized"
        )
    }

    #[test]
    fn test_resampler_process() {
        let input_layout = AudioChannelLayout::new(1).expect("Failed to create new AudioChannelLayout");
        let input_format = AVSampleFormat::S16;
        let input_sample_rate = 44100;

        let output_layout = AudioChannelLayout::new(2).expect("Failed to create new AudioChannelLayout");
        let output_format = AVSampleFormat::S16p;
        let output_sample_rate = 48000;

        let mut resampler = Resampler::new(
            input_layout.copy().unwrap(),
            input_format,
            input_sample_rate,
            output_layout,
            output_format,
            output_sample_rate,
        )
        .expect("Failed to create new Resampler");

        let mut input_frame = AudioFrame::builder()
            .nb_samples(1024)
            .channel_layout(input_layout)
            .sample_fmt(input_format)
            .sample_rate(44100)
            .build()
            .expect("Failed to create input AudioFrame");

        let input_data = input_frame.data_mut(0).expect("Data buffer of input frame was invalid");
        rng().fill(input_data);

        let output = resampler.process(&input_frame).expect("Failed to process frame");

        assert_eq!(output.channel_count(), 2, "Output channel count should be 2");
        assert!(output.data(0).is_some(), "First data buffer of output frame is None");
        assert!(output.data(1).is_some(), "Second data buffer of output frame is None");
        assert_eq!(output.sample_rate(), 48000, "Output sample rate was not 48000");
    }
}
