use std::error::Error;
use std::path::Path;

use druid::image::io::Reader as ImageReader;
use druid::widget::{Controller, Image};
use druid::{Env, ImageBuf, LifeCycle, LifeCycleCtx, UpdateCtx, Widget};
use tracing::debug;

pub struct UIImageController;

impl UIImageController {
    fn get_image_buf(&self, icon_path: &str) -> Result<ImageBuf, Box<dyn Error>> {
        if icon_path.is_empty() {
            return Ok(ImageBuf::empty());
        }

        let path1 = Path::new(icon_path);

        let dynamic_image = ImageReader::open(path1)?.decode()?;
        let buf = ImageBuf::from_dynamic_image(dynamic_image);
        return Ok(buf);
    }
}

impl Controller<String, Image> for UIImageController {
    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        icon_path: &String,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                debug!("WidgetAdded WAS CALLED for icon {}", icon_path.clone());
                if let Ok(buf) = self.get_image_buf(icon_path.as_str()) {
                    child.set_image_data(buf);
                }
            }
            _ => {
                // TODO: check if icon path has changed
                //info!("other event {:?} for icon {}", event, icon_path.clone());
            }
        }

        child.lifecycle(ctx, event, icon_path, env)
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut UpdateCtx,
        old_icon_path: &String,
        icon_path: &String,
        _env: &Env,
    ) {
        if icon_path != old_icon_path {
            debug!(
                "icon changed from {} to {}",
                old_icon_path.clone(),
                icon_path.clone()
            );

            if let Ok(buf) = self.get_image_buf(icon_path.as_str()) {
                child.set_image_data(buf);
                ctx.children_changed();
            }
        }
    }
}
