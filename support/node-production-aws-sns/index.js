const { ApiPromise, WsProvider } = require('@polkadot/api');
const AWS = require('aws-sdk');
const Settings = require('./settings.json')

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress) {
    const provider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ provider });
    await api.isReady;
    return api;
}

async function main() {
    const api = await createPromiseApi(Settings.nodeAddress);

    AWS.config.update({region: Settings.region});
    const sns = new AWS.SNS({apiVersion: Settings.awsApiVersion});
    const topics = await sns.listTopics().promise();
    const topicArn = topics.Topics.find(t => t.TopicArn.includes(Settings.topicName)).TopicArn;

    let lastHeaderHex = Settings.lastHeaderHex;
    let lastNewBlockAt = Date.now();
    let sendingSns = false;
    let sleepTime = 0;
    while (true) {
        await new Promise(r => setTimeout(r, sleepTime));
        try {
            sleepTime = Settings.sleepIntervalMs
            const signedBlock = await api.rpc.chain.getBlock();
            const currentHeader = signedBlock.block.header.hash;

            if (currentHeader.toHex() != lastHeaderHex) {
                lastHeaderHex = currentHeader.toHex();
                lastNewBlockAt = Date.now();
                console.log(`Block header hex: ${lastHeaderHex} at ${new Date().toISOString()}`);
                continue;
            }

            console.log(new Date().toISOString(), `Have not seen new block for ${new Date() - lastNewBlockAt}ms`);

            if (sendingSns) {
                continue;
            }

            if (Date.now() - lastNewBlockAt > Settings.requireNewBlockEveryMs) {
                sleepTime = 1000 * 60 * Settings.minutesUntilNextCheckAfterAlarm;
                sendingSns = true;
                let data = await sns.publish({
                    Message: `message: ${Settings.message};  Last header hex: ${lastHeaderHex}`,
                    TopicArn: topicArn,
                }).promise();
                console.warn(`Message ${Settings.message} sent to the topic ${topicArn}`);
                console.warn("MessageID is " + data.MessageId);
                sendingSns = false;
            }
        } catch (err) {
            sendingSns = false;
            lastNewBlockAt = Date.now();
            sleepTime = Settings.sleepIntervalMs;
            console.error(err, err.stack);
        }
    };
}

main().catch(console.error);
