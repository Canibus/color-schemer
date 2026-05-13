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
        // Pattern: {productName}_{version}_x64_en-US.msi.zip.sig
        const sigFileName = `${productName}_${version}_x64_en-US.msi.zip.sig`;
        const sigFilePath = path.join(BUNDLE_DIR, sigFileName);

        let signature = process.env.SIGNATURE;
        if (!signature && fs.existsSync(sigFilePath)) {
            console.log(`Reading signature from ${sigFilePath}`);
            signature = fs.readFileSync(sigFilePath, 'utf8').trim();
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
