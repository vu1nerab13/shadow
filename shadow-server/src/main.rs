/*!
 * *Shadow RAT*
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 *
 * 2024.6.13
 */

use anyhow::Result as AppResult;
use clap::Parser;
use flexi_logger::Logger;
use log::debug;
use shadow_server::{network, web, AppArgs};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Main entry point
#[tokio::main]
async fn main() -> AppResult<()> {
    // Parse arguments
    let args = AppArgs::parse();

    // Start logging
    #[cfg(debug_assertions)]
    Logger::try_with_str(args.verbose)?.start()?;

    debug!(
        "server address: {}, web address: {}",
        args.server_addr, args.web_addr
    );

    // A instance representing all clients connected to the server
    let server_objs = Arc::new(RwLock::new(HashMap::new()));
    // Server config
    let server_cfg =
        network::Config::new(args.server_addr.parse()?, args.cert_path, args.pri_key_path);
    // Web interface config
    let web_cfg = web::Config::new(args.web_addr.parse()?);

    // Start the server
    let server = tokio::spawn(network::run(server_cfg, server_objs.clone()));

    // Start web interface
    tokio::spawn(web::run(web_cfg, server_objs));

    // Wait until server shutdown
    server.await?
}
