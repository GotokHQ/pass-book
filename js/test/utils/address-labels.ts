import { AddressLabels } from '@metaplex-foundation/amman';
import { logDebug } from '.';

const persistLabelsPath = process.env.ADDRESS_LABEL_PATH;
const knownLabels = { ['passK9sjcBkUzWu35gf2x4EmpcrkZB9NXgHWtgAzxhB']: 'mpl-token-metadata' };

const addressLabels = new AddressLabels(knownLabels, logDebug, persistLabelsPath);
export const addLabel = addressLabels.addLabel;
export const isKeyOf = addressLabels.isKeyOf;
