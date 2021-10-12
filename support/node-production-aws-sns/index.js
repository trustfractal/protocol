const { ApiPromise, WsProvider } = require('@polkadot/api');
const AWS = require('aws-sdk');
const nodeCron = require('node-cron');
const settings = require('./settings.json')

// Create a promise API instance of the passed in node address.
async function createPromiseApi(nodeAddress, types) {
    const provider = new WsProvider(nodeAddress);
    const api = await new ApiPromise({ provider, types });
    await api.isReady;
    return api;
}

async function publishToTopic(topicName, message) {
    AWS.config.update({region: settings.region});
    const sns = new AWS.SNS({apiVersion: settings.awsApiVersion});
    const topics = await sns.listTopics().promise();
    const topicArn = topics.Topics.find(t => t.TopicArn.includes(topicName)).TopicArn;

    return sns.publish({
        Message: message,
        TopicArn: topicArn,
    }).promise();
}

async function main() {
    const api = await createPromiseApi(settings.nodeAddress);

    let lastHeaderHex = settings.lastHeaderHex;
    let counter = 0;
    let sendingSns = false;
    const job = nodeCron.schedule(settings.cronTime, () => {
        console.log(new Date().toLocaleString());
        api.rpc.chain.getBlock().then(signedBlock => {
            const currentHeader = signedBlock.block.header.hash;
            if (currentHeader.toHex() != lastHeaderHex) {
                lastHeaderHex = currentHeader.toHex();
                counter = 0;
            } else {
                counter++;
                if (counter > 1 && !sendingSns) {
                    sendingSns = true;

                    // Create promise and SNS service object
                    publishToTopic(settings.topicName, `message: ${settings.message};  Last header hex: ${lastHeaderHex}`).then((data) => {
                        sendingSns = false;
                        counter = 0;
                        console.log(`Message ${settings.message} sent to the topic ${settings.topicName}`);
                        console.log("MessageID is " + data.MessageId);
                        //TODO: save the latest header hex to the settings.json
                    }).catch((err) => {
                        sendingSns = false;
                        counter = 0;
                        console.error(err, err.stack);
                    });
                }
            }
        });
    });
}

main().catch(console.error);
