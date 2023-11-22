use std::mem::MaybeUninit;

use windows::{
    core::Result,
    core::{implement, HSTRING, PCWSTR},
    Win32::Media::MediaFoundation::{
        IMFAsyncCallback, IMFAsyncCallback_Impl, IMFAsyncResult, IMFMediaSession, MESessionClosed,
        MFCreateMediaSession, MFCreateSourceResolver, MFShutdown, MFStartup, MFSTARTUP_FULL,
        MF_OBJECT_TYPE, MF_RESOLUTION_MEDIASOURCE, MF_VERSION,
    },
};

fn main() {
    unsafe {
        // Start Media foundation
        match MFStartup(MF_VERSION, MFSTARTUP_FULL) {
            Ok(_) => (),
            Err(_) => println!("Error starting MF"),
        }

        play_music();

        // End Media foundation
        match MFShutdown() {
            Ok(_) => (),
            Err(_) => println!("Error shutting down MF"),
        }
    }
}

fn play_music() {
    // 1. Create a new media session.
    // 2. Create the media source.
    // 3. Create the topology.
    // 4. Queue the topology [asynchronous]
    // 5. Start playback [asynchronous - does not happen in this method.]

    let player = Player::new();
    // Open media source
}

#[allow(unused)]
enum PlayerState {
    Closed,
    Ready,
    OpenPending,
    Started,
    Paused,
    Stopped,
    Closing,
}

#[allow(non_snake_case)]
#[implement(windows::Win32::Media::MediaFoundation::IMFAsyncCallback)]
struct Player {
    session: IMFMediaSession,
    state: PlayerState,
}

impl Player {
    fn new() -> Result<Self> {
        // TODO: Close any previous sessions
        let session = unsafe { MFCreateMediaSession(None)? };
        let player = Player {
            session,
            state: PlayerState::Ready,
        };
        // Start pulling events from the media session
        unsafe {
            player
                .session
                .BeginGetEvent(&player.cast::<IMFAsyncCallback>().unwrap(), None)?
        };

        Ok(player)
    }

    fn play_music(url: &str) -> Result<()> {
        // create media source
        let source_resolver = unsafe { MFCreateSourceResolver()? };
        let mut obj_type = MF_OBJECT_TYPE::default();
        let mut source = MaybeUninit::uninit();
        // NOTE: For the PCWSTR : https://github.com/microsoft/windows-rs/issues/973#issuecomment-1346528303
        unsafe {
            source_resolver.CreateObjectFromURL(
                PCWSTR(HSTRING::from(url).as_ptr()),
                MF_RESOLUTION_MEDIASOURCE.0 as u32,
                None,
                &mut obj_type,
                source.as_mut_ptr(),
            )
        };

        Ok(())
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        //TODO: Close session
    }
}

#[allow(non_snake_case)]
impl IMFAsyncCallback_Impl for Player {
    fn GetParameters(&self, pdwflags: *mut u32, pdwqueue: *mut u32) -> Result<()> {
        Ok(())
    }

    fn Invoke(&self, pasyncresult: Option<&IMFAsyncResult>) -> Result<()> {
        // 1) In the Invoke method, the CPlayer object posts a WM_APP_PLAYER_EVENT message to the application. The message parameter is an IMFMediaEvent pointer.
        // 2) The application receives the WM_APP_PLAYER_EVENT message.
        // 3) The application calls the CPlayer::HandleEvent method, passing in the IMFMediaEvent pointer.
        // 4) The HandleEvent method responds to the event.
        //
        // 1) Call IMFMediaEventGenerator::EndGetEvent to get the event. This method returns a pointer to the IMFMediaEvent interface.
        // 2) Call IMFMediaEvent::GetType to get the event code.
        // 3) If the event code is MESessionClosed, call SetEvent to set the m_hCloseEvent event. The reason for this step is explained in Step 7: Shut Down the Media Session, and also in the code comments.
        // 4) For all other event codes, call IMFMediaEventGenerator::BeginGetEvent to request the next event.
        // 5) Post a WM_APP_PLAYER_EVENT message to the window.

        let media_event = unsafe { self.session.EndGetEvent(pasyncresult.unwrap())? };
        match unsafe { media_event.GetType() } {
            Ok(x) if x == MESessionClosed.0 as u32 => {
                // Should close session here
            }
            Ok(_) => unsafe {
                self.session
                    .BeginGetEvent(&self.cast::<IMFAsyncCallback>().unwrap(), None);
            },
            Err(_) => panic!("Huh?"),
        };

        Ok(())
    }
}
