use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::string::String;

use clap::parser::ValueSource::CommandLine;
use clap::ArgMatches;
use log::Level::Debug;
use log::{debug, error, info, log_enabled, warn};
use zune_image::codecs::ImageFormat;
use zune_image::errors::ImageErrors;
use zune_image::traits::IntoImage;
use zune_image::workflow::WorkFlow;

use crate::cmd_parsers::get_decoder_options;
use crate::cmd_parsers::global_options::CmdOptions;
use crate::file_io::ZuneFile;
use crate::probe_files::probe_input_files;
use crate::show_gui::open_in_default_app;
use crate::MmapOptions;

#[allow(unused_variables)]
#[allow(clippy::unused_io_amount)] // yes it's what I want
pub(crate) fn create_and_exec_workflow_from_cmd(
    args: &ArgMatches, options: &[String], cmd_opts: &CmdOptions
) -> Result<(), ImageErrors>
{
    if let Some(view) = args.value_source("probe")
    {
        if view == CommandLine
        {
            probe_input_files(args);
            return Ok(());
        }
    }

    info!("Creating workflows from input");

    let decoder_options = get_decoder_options(args);
    let mut buf = [0; 30];

    for in_file in args.get_raw("in").unwrap()
    {
        let mut workflow: WorkFlow<ZuneFile> = WorkFlow::new();

        File::open(in_file)?.read(&mut buf)?;

        add_operations(args, options, &mut workflow)?;

        let mmap_opt = cmd_opts.mmap;
        let use_mmap = mmap_opt == MmapOptions::Auto || mmap_opt == MmapOptions::Always;

        if let Some(format) = ImageFormat::guess_format(&buf)
        {
            if format.has_decoder()
            {
                workflow.add_decoder(ZuneFile::new(
                    in_file.to_os_string(),
                    use_mmap,
                    decoder_options
                ))
            }
            else
            {
                return Err(ImageErrors::ImageDecoderNotImplemented(format));
            }
        }
        else
        {
            return Err(ImageErrors::ImageDecoderNotIncluded(ImageFormat::Unknown));
        }

        if let Some(source) = args.value_source("out")
        {
            if source == CommandLine
            {
                for out_file in args.get_raw("out").unwrap()
                {
                    if let Some(ext) = Path::new(out_file).extension()
                    {
                        if let Some((encode_type, encoder)) =
                            ImageFormat::get_encoder_for_extension(ext.to_str().unwrap())
                        {
                            debug!("Treating {:?} as a {:?} format", out_file, encode_type);
                            workflow.add_encoder(encoder);
                        }
                        else
                        {
                            error!("Unknown or unsupported format {:?}", out_file)
                        }
                    }
                    else
                    {
                        error!("Could not determine extension from {:?}", out_file)
                    }
                }
            }
        }

        workflow.advance_to_end()?;
        let results = workflow.get_results();
        let mut curr_result_position = 0;

        // write to output

        //  We support multiple format writes per invocation
        // i.e it's perfectly valid to do -o a.ppm , -o a.png
        if let Some(source) = args.value_source("out")
        {
            if source == CommandLine
            {
                for out_file in args.get_raw("out").unwrap()
                {
                    //write to file
                    if let Some(ext) = Path::new(out_file).extension()
                    {
                        if let Some((encode_type, _)) =
                            ImageFormat::get_encoder_for_extension(ext.to_str().unwrap())
                        {
                            if encode_type.has_encoder()
                                && results[curr_result_position].get_format() == encode_type
                            {
                                info!(
                                    "Writing data as {:?} format to file {:?}",
                                    results[curr_result_position].get_format(),
                                    out_file
                                );

                                std::fs::write(out_file, results[curr_result_position].get_data())
                                    .unwrap();

                                curr_result_position += 1;
                            }
                            else
                            {
                                warn!("Ignoring {:?} file", out_file);
                            }
                        }
                        else
                        {
                            warn!("Ignoring {:?} file", out_file);
                        }
                    }
                }
            }
        }

        if let Some(view) = args.value_source("view")
        {
            if view == CommandLine
            {
                for image in workflow.get_images()
                {
                    open_in_default_app(image);
                }
            }
        }
    }

    Ok(())
}

pub fn add_operations<T: IntoImage>(
    args: &ArgMatches, order_args: &[String], workflow: &mut WorkFlow<T>
) -> Result<(), String>
{
    if log_enabled!(Debug) && args.value_source("operations") == Some(CommandLine)
    {
        println!();
    }

    crate::cmd_parsers::operations::parse_options(workflow, order_args, args)?;
    crate::cmd_parsers::filters::parse_options(workflow, order_args, args)?;

    debug!("Arranging options as specified in cmd");

    if log_enabled!(Debug) && args.value_source("operations") == Some(CommandLine)
    {
        println!();
    }
    Ok(())
}
