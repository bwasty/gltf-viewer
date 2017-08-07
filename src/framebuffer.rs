use std::ptr;

use gl;

#[derive(Debug)]
struct Framebuffer {
    pub id: u32
}

impl Framebuffer {
    pub unsafe fn new(width: u32, height: u32) -> Framebuffer {
        let mut framebuffer = 0;
        gl::GenFramebuffers(1, &mut framebuffer);
        gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
        // create a color attachment texture
        let mut texture_colorbuffer = 0;
        gl::GenTextures(1, &mut texture_colorbuffer);
        gl::BindTexture(gl::TEXTURE_2D, texture_colorbuffer);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, width as i32, height as i32,
            0, gl::RGB, gl::UNSIGNED_BYTE, ptr::null());
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_colorbuffer, 0);
        // create a renderbuffer object for depth and stencil attachment (we won't be sampling these)
        let mut rbo = 0;
        gl::GenRenderbuffers(1, &mut rbo);
        gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
        gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width as i32, height as i32); // use a single renderbuffer object for both a depth AND stencil buffer.
        gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::DEPTH_STENCIL_ATTACHMENT, gl::RENDERBUFFER, rbo); // now actually attach it
        // now that we actually created the framebuffer and added all attachments we want to check if it is actually complete now
        if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
            panic!("ERROR::FRAMEBUFFER:: Framebuffer is not complete!");
        }
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

        Framebuffer {
            id: framebuffer
        }
    }

    pub unsafe fn bind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
    }

    pub unsafe fn unbind(&self) {
        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
    }
}
