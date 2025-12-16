import { Address, address, getAddressEncoder, getProgramDerivedAddress } from '@solana/kit';
import { VAULT_PROGRAM_ID, USER_VAULT_SEED, USER_VAULT_LAMPORTS_SEED } from './constants';

const userVaultSeed = Buffer.from(USER_VAULT_SEED);
const userVaultLamportsSeed = Buffer.from(USER_VAULT_LAMPORTS_SEED);


export async function getUserVaultPda(user: string) {
    return getProgramDerivedAddress({
        programAddress: VAULT_PROGRAM_ID,
        seeds: [
            userVaultSeed,
            getAddressEncoder().encode(address(user)),
        ],
    });
}

export async function getUserVaultLamportsPda(user: string) {
    return getProgramDerivedAddress({
        programAddress: VAULT_PROGRAM_ID,
        seeds: [
            userVaultLamportsSeed,
            getAddressEncoder().encode(address(user)),
            getAddressEncoder().encode((await getUserVaultPda(user))[0])
        ],
    });
}