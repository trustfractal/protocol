export declare function SetupSuccess({ onContinue, mnemonic, }: {
    mnemonic: string;
    onContinue: () => void;
}): JSX.Element;
export declare function SetupError({ onRetry }: {
    onRetry: () => void;
}): JSX.Element;
export declare function SetupInProgress({ onRetry }: {
    onRetry: () => void;
}): JSX.Element;
