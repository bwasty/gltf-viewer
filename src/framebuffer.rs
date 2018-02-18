use std::ptr;

use gl;

#[derive(Debug)]
pub struct Framebuffer {
    pub id: u32
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Framebuffer {
        let mut framebuffer = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            // create a color attachment texture
            let mut texture_colorbuffer = 0;
            gl::GenTextures(1, &mut texture_colorbuffer);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, texture_colorbuffer);
            gl::TexImage2DMultisample(gl::TEXTURE_2D_MULTISAMPLE, 4, gl::RGBA as u32, width as i32, height as i32,
                gl::TRUE);
            // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            // gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::BindTexture(gl::TEXTURE_2D_MULTISAMPLE, 0);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_colorbuffer, 0);
            // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
            let mut rbo = 0;
            gl::GenRenderbuffers(1, &mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::DEPTH24_STENCIL8, width as i32, height as i32); // use a single renderbuffer object for both a depth AND stencil buffer.
            gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
            gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo); // now actually attach it
            // now that we actually created the framebuffer and added all attachments we want to check if it is actually complete now
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        Framebuffer {
            id: framebuffer
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, self.id) }
    }

    pub fn unbind(&self) {
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0) }
    }
}
