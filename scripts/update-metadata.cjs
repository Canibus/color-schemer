const fs = require('fs');
const path = require('path');

// Configuration
const TAURI_CONF_PATH = 'src-tauri/tauri.conf.json';
const BUNDLE_DIR = 'src-tauri/target/release/bundle/msi';

function main() {
    try {
        // Read tauri.conf.json
        const tauriConf = JSON.parse(fs.readFileSync(TAURI_CONF_PATH, 'utf8'));
        const productName = tauriConf.package.productName;
        const version = process.env.VERSION || tauriConf.package.version;

        console.log(`Generating metadata for ${productName} v${version}`);

        // Find signature file
        let signature = process.env.SIGNATURE;
        
        if (!signature) {
            const bundlePaths = [
                path.join('src-tauri/target/release/bundle/msi', `${productName}_${version}_x64_en-US.msi.zip.sig`),
                path.join('src-tauri/target/release/bundle/nsis', `${productName}_${version}_x64-setup.exe.sig`)
            ];

            for (const sigFilePath of bundlePaths) {
                if (fs.existsSync(sigFilePath)) {
                    console.log(`Reading signature from ${sigFilePath}`);
                    signature = fs.readFileSync(sigFilePath, 'utf8').trim();
                    break;
                }
            }
        }
        
        // Fallback: search for any .sig file in the bundle directory if standard paths fail
        if (!signature) {
            const baseBundleDir = 'src-tauri/target/release/bundle';
            if (fs.existsSync(baseBundleDir)) {
                const findSig = (dir) => {
                    const files = fs.readdirSync(dir);
                    for (const file of files) {
                        const fullPath = path.join(dir, file);
                        if (fs.statSync(fullPath).isDirectory()) {
                            const found = findSig(fullPath);
                            if (found) return found;
                        } else if (file.endsWith('.sig')) {
                            return fullPath;
                        }
                    }
                    return null;
                };
                
                const foundSigFile = findSig(baseBundleDir);
                if (foundSigFile) {
                    console.log(`Found fallback signature at ${foundSigFile}`);
                    signature = fs.readFileSync(foundSigFile, 'utf8').trim();
                }
            }
        }

        if (!signature) {
            console.error('Error: No signature found. Set SIGNATURE env var or ensure .sig file exists.');
            process.exit(1);
        }

        const url = process.env.URL || `https://github.com/canibus/color-schemer/releases/download/v${version}/${productName}_${version}_x64_en-US.msi.zip`;

        const metadata = {
            version: version,
            notes: `Release v${version}`,
            pub_date: new Date().toISOString(),
            platforms: {
                "windows-x86_64": {
                    signature: signature,
                    url: url
                }
            }
        };

        fs.writeFileSync('update.json', JSON.stringify(metadata, null, 2));
        console.log('Successfully generated update.json');
    } catch (error) {
        console.error('Failed to generate metadata:', error.message);
        process.exit(1);
    }
}

main();
